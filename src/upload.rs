//! Specific implementation of operations related to application update.

use crate::ssh_config::SshConfig;
use crate::util::{
    get_session,
    wait_exec,
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
use ssh::{ WRITE, RECURSIVE };

//------------------------------------------------------------------------------
/// Uploads all files in the local repository to the remote server.
//------------------------------------------------------------------------------
pub fn upload_all
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

    let mut session = get_session(project);
    {
        let mut channel = session.channel_new().unwrap();
        channel.open_session().unwrap();
        let command = format!("rm -r {}/*", &remote_target_path);
        channel.request_exec(command.as_bytes()).unwrap();
        channel.send_eof().unwrap();
        channel.close();
    }
    println!("Removed.\n");

    //  Uploads all files.
    println!("Upload files.");
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
        upload
        (
            project,
            &Path::new(&temp_file),
            &Path::new(&remote_path_str),
        );

        let mut session = get_session(project);
        {
            let mut channel = session.channel_new().unwrap();
            channel.open_session().unwrap();
            let command = format!
            (
                "unzip -o {} -d {} && rm {}",
                &remote_path_str,
                &remote_path,
                &remote_path_str,
            );
            channel.request_exec(command.as_bytes()).unwrap();
            channel.send_eof().unwrap();
            channel.close();
        }
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

            upload(project, &path, &Path::new(&remote_path_str));
        }
    }

    println!("Done.\n");
    clear_cache(project, &remote_cache_path);
    println!("Done.");
}

//------------------------------------------------------------------------------
/// Uploads only specified files in the local repository to the remote server.
//------------------------------------------------------------------------------
pub fn upload_patch
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

    //  Uploads files.
    for path in paths
    {
        let path_str = path.to_str().unwrap();
        let remote_path_str = remote_path.clone()
            + &path_str.replace(&git_src_path, "");
        upload(project, &Path::new(path_str), &Path::new(&remote_path_str));
    }

    clear_cache(&project, &remote_cache_path);
    println!("Done.");
}

//------------------------------------------------------------------------------
/// Uploads file to the remote server.
//------------------------------------------------------------------------------
fn upload( project: &str, from: &Path, to: &Path )
{
    println!
    (
        "{} : {} => {}",
        "Uploading".green(),
        from.display(),
        to.display(),
    );

    if from.is_dir()
    {
        let mut session = get_session(project);
        let mut scp = session
            .scp_new(WRITE|RECURSIVE, to.to_str().unwrap())
            .unwrap();
        scp.init().unwrap();
        scp.push_directory(to, 0o755).unwrap();
        scp.close();
    }
    else
    {
        mkdir_all(project, to.parent().unwrap());

        let mut file = File::open(from).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        let mut session = get_session(project);
        let mut scp = session
            .scp_new(WRITE|RECURSIVE, to.to_str().unwrap())
            .unwrap();
        scp.init().unwrap();
        scp.push_file(to, buf.len(), 0o644).unwrap();

        let mut pos = 0;
        while pos < buf.len()
        {
            let mut len = buf.len() - pos;
            if len > 1024
            {
                len = 1024;
            }
            scp.write(&buf[pos..pos+len]).unwrap();
            pos += len;
        }
        scp.close();
    }
}

//------------------------------------------------------------------------------
/// Creates all directories in the path.
//------------------------------------------------------------------------------
fn mkdir_all( project: &str, path: &Path )
{
    let mut session = get_session(project);
    {
        let mut channel = session.channel_new().unwrap();
        channel.open_session().unwrap();
        let command = format!("mkdir -p {}", path.to_str().unwrap());
        channel.request_exec(command.as_bytes()).unwrap();
        wait_exec(&channel);
        channel.send_eof().unwrap();
        channel.close();
    }
}

//------------------------------------------------------------------------------
/// Clears the cache on the remote server.
//------------------------------------------------------------------------------
pub fn clear_cache( project: &str, remote_cache_path: &str )
{
    print_sep();
    println!("{} : {}", "Remote cache path".green(), remote_cache_path);
    print_sep();
    if confirm("Delete the cache for the above path?")
    {
        let mut session = get_session(project);
        {
            let mut channel = session.channel_new().unwrap();
            channel.open_session().unwrap();
            let command = format!("rm -r {}/*", remote_cache_path);
            channel.request_exec(command.as_bytes()).unwrap();
            channel.send_eof().unwrap();
            channel.close();
        }
    }
}
