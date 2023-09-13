//! Specific implementation of operations related to application update.

use crate::ssh_config::SshConfig;
use crate::util::{
    get_session,
    get_sftp_session,
    get_file_paths,
    print_sep,
    confirm,
};

use std::path::{ Path, PathBuf };
use std::time::SystemTime;
use std::fs::File;
use std::io::{ Read, Write };

use colored::Colorize;
use git2::{ Repository, StatusOptions };
use zip::write::{ ZipWriter, FileOptions };
use openssh::Session;
use openssh_sftp_client::fs::Fs;
use async_recursion::async_recursion;

//------------------------------------------------------------------------------
/// Uploads all files in the local repository to the remote server.
//------------------------------------------------------------------------------
pub async fn upload_all
(
    project: &str,
    config: &SshConfig,
    target_path: String,
    zip: bool,
)
{
    let git_src_path = config.git_src_path();
    let remote_path = config.remote_path();
    let mut remote_target_path = config.remote_path();
    if target_path.len() > 0
    {
        remote_target_path = remote_target_path + "/" + &target_path;
    }
    let remote_cache_path = config.remote_cache_path();

    //  Removes the remote directory.
    print_sep();
    println!("{} : {}", "Project    ".green(), &config.project());
    println!("{} : {}", "Environment".green(), &config.environment());
    println!("{} : {}", "Remote path".green(), &remote_path);
    println!("{} : {}", "target path".green(), &target_path);
    print_sep();
    if confirm("Are you sure you want to remove the remote directory?") == false
    {
        println!("Canceled.");
        return;
    }

    let session = get_session(project).await;
    session
        .command("rm")
        .args(["-r", &remote_target_path])
        .output()
        .await
        .unwrap();

    println!("Removed.\n");
    println!("Upload files.");

    //  Uploads all files.
    let sftp = get_sftp_session(project).await;
    {
        let mut fs = sftp.fs();
        mkdir_all(&mut fs, &Path::new(&remote_target_path)).await;
        let paths = get_file_paths(&git_src_path);

        if zip
        {
            //  Creates a temporary zip file.
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let temp_file = "temp_".to_string() + &now.to_string() + ".zip";

            let zip_file = File::create(&temp_file).unwrap();
            let mut zip = ZipWriter::new(zip_file);
            for path in paths
            {
                let path_str = path.to_str().unwrap();

                //  Skip if the target path is specified and the path is not in
                //  the target path.
                if target_path.len() > 0
                {
                    let target_path = git_src_path.clone() + "/" + &target_path;
                    if path_str.starts_with(&target_path) == false
                    {
                        continue;
                    }
                }

                //  Adds the file to the zip file.
                let path_str = path_str.replace(&git_src_path, "");
                zip.start_file(path_str, FileOptions::default()).unwrap();
                let content = std::fs::read(path.to_str().unwrap()).unwrap();
                zip.write_all(&content).unwrap();
            }
            zip.finish().unwrap();

            //  Uploads the zip file.
            let remote_path_str = remote_path.clone() + "/" + &temp_file;
            upload(&mut fs, &temp_file, &remote_path_str, false).await;

            session
                .command("unzip")
                .args(["-o", &remote_path_str, "-d", &remote_path])
                .output()
                .await
                .unwrap();
            session
                .command("rm")
                .args(["-r", &remote_path_str])
                .output()
                .await
                .unwrap();
            std::fs::remove_file(&temp_file).unwrap();
        }
        else
        {
            for path in paths
            {
                let path_str = path.to_str().unwrap();
                let remote_path_str = remote_path.clone()
                    + &path_str.replace(&git_src_path, "");

                //  Skip if the target path is specified and the path is not in
                //  the target path.
                if target_path.len() > 0
                {
                    let target_path = git_src_path.clone() + "/" + &target_path;
                    if path_str.starts_with(&target_path) == false
                    {
                        continue;
                    }
                }

                upload(&mut fs, path_str, &remote_path_str, path.is_dir())
                    .await;
            }
        }
    }
    sftp.close().await.unwrap();

    println!("Done.\n");
    clear_cache(&session, &remote_cache_path).await;
    session.close().await.unwrap();
    println!("Done.");
}

//------------------------------------------------------------------------------
/// Uploads only specified files in the local repository to the remote server.
//------------------------------------------------------------------------------
pub async fn upload_patch
(
    project: &str,
    config: &SshConfig,
    patch_file: String,
)
{
    let git_src_path = config.git_src_path();
    let remote_path = config.remote_path();
    let remote_cache_path = config.remote_cache_path();

    let mut paths = Vec::new();
    if patch_file.len() > 0
    {
        //  Reads the patch file.
        let patch_file = std::fs::read_to_string(patch_file).unwrap();
        for line in patch_file.lines()
        {
            let path = git_src_path.clone() + "/" + line;
            paths.push(PathBuf::from(path));
        }
    }
    else
    {
        //  Gets the list of files from the git repository.
        let git_path = config.git_path();
        let git_relative_path = config.get_git_relative_path();
        let repo = Repository::open(&git_path).unwrap();
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut status_opts)).unwrap();

        for status in statuses.iter()
        {
            let path = status.path().unwrap();
            if path.starts_with(&git_relative_path) == false
            {
                continue;
            }
            paths.push(PathBuf::from(&(git_path.clone() + "/" + path)));
        }
    }

    if paths.len() == 0
    {
        println!("No files to upload.");
        return;
    }

    print_sep();
    println!("{} : {}", "Project    ".green(), &config.project());
    println!("{} : {}", "Environment".green(), &config.environment());
    println!("{} : {}", "Remote path".green(), &remote_path);
    println!("{} :",    "Files      ".green());
    for path in &paths
    {
        println!("    - {}", path.to_str().unwrap());
    }
    print_sep();
    if confirm("Are you sure you want to upload these files?") == false
    {
        println!("Canceled.");
        return;
    }

    //  Uploads all files.
    let sftp = get_sftp_session(project).await;
    {
        let mut fs = sftp.fs();
        for path in paths
        {
            let path_str = path.to_str().unwrap();
            let remote_path_str = remote_path.clone()
                + &path_str.replace(&git_src_path, "");
            upload(&mut fs, path_str, &remote_path_str, path.is_dir()).await;
        }
    }
    sftp.close().await.unwrap();

    println!("Done.\n");
    let session = get_session(project).await;
    clear_cache(&session, &remote_cache_path).await;
    session.close().await.unwrap();
    println!("Done.");
}

//------------------------------------------------------------------------------
/// Uploads file to the remote server.
//------------------------------------------------------------------------------
async fn upload( fs: &mut Fs, from: &str, to: &str, is_dir: bool )
{
    println!
    (
        "{} : {} => {}",
        "Uploading".green(),
        from,
        to,
    );

    if is_dir
    {
        fs.create_dir(to).await.unwrap();
    }
    else
    {
        if fs.metadata(Path::new(to).parent().unwrap()).await.is_err()
        {
            mkdir_all(fs, &Path::new(to).parent().unwrap()).await;
        }

        if let Ok(mut file) = File::open(from)
        {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();
            fs.write(to, &buf).await.unwrap();
        }
        else
        {
            println!("Skip to upload the file.");
        }
    }
}

//------------------------------------------------------------------------------
/// Creates all directories in the path.
//------------------------------------------------------------------------------
#[async_recursion]
async fn mkdir_all( fs: &mut Fs, path: &Path )
{
    if fs.metadata(path.parent().unwrap()).await.is_err()
    {
        mkdir_all(fs, path.parent().unwrap()).await;
    }
    fs.create_dir(path).await.unwrap();
}

//------------------------------------------------------------------------------
/// Clears the cache on the remote server.
//------------------------------------------------------------------------------
pub async fn clear_cache( session: &Session, remote_cache_path: &str )
{
    print_sep();
    println!("{} : {}", "Remote cache path".green(), remote_cache_path);
    print_sep();
    if confirm("Delete the cache for the above path?")
    {
        session
            .command("rm")
            .args(["-r", remote_cache_path])
            .output()
            .await
            .unwrap();
        session
            .command("mkdir")
            .args([remote_cache_path, "-m", "777"])
            .output()
            .await
            .unwrap();
    }
}
