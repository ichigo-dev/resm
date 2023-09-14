//! Specific implementation of operations related to application backup.

use crate::ssh_config::SshConfig;
use crate::util::{
    get_session,
    get_sftp_session,
    print_sep,
    get_current_time_for_filename,
};

use std::fs::File;
use std::io::Write;

use colored::Colorize;

//------------------------------------------------------------------------------
/// Backs up the remote directory.
///
/// # Arguments
///
/// - `project` - Project name.
/// - `config` - SSH configuration.
/// - `target_path` - Relative path from the project directory that you want to
/// upload.
//------------------------------------------------------------------------------
pub async fn backup
(
    project: &str,
    config: &SshConfig,
    target_path: String,
)
{
    let backup_path = config.backup_path();
    let mut remote_path = config.remote_path();
    if target_path.len() > 0
    {
        remote_path = remote_path + "/" + &target_path;
    }

    let now = get_current_time_for_filename();
    let backup_file = if target_path.len() > 0
    {
        format!("{}_{}_{}.zip", project, &target_path, &now)
    }
    else
    {
        format!("{}_{}.zip", project, &now)
    };
    let backup_path = backup_path + "/" + &backup_file;

    print_sep();
    println!("{} : {}", "Project    ".green(), &config.project());
    println!("{} : {}", "Environment".green(), &config.environment());
    println!("{} : {}", "Backup path".green(), &backup_path);
    println!("{} : {}", "Remote path".green(), &remote_path);
    print_sep();
    println!("Exporting...\n");

    //  Gets the backup file.
    let session = get_session(project).await;
    session
        .command("zip")
        .args(["-r", &backup_file, &remote_path])
        .output()
        .await
        .unwrap();
    let sftp = get_sftp_session(project).await;
    {
        let mut fs = sftp.fs();
        let content = fs.read(&backup_file).await.unwrap();
        let mut file = File::create(&backup_path).unwrap();
        file.write_all(&content).unwrap();
    }
    sftp.close().await.unwrap();
    session
        .command("rm")
        .arg(&backup_file)
        .output()
        .await
        .unwrap();
    session.close().await.unwrap();
    println!("Done.");
}

//------------------------------------------------------------------------------
/// Backs up the database.
///
/// # Arguments
///
/// - `project` - Project name.
/// - `config` - SSH configuration.
/// - `target_tables` - Tables to be backed up.
//------------------------------------------------------------------------------
pub async fn backup_db
(
    project: &str,
    config: &SshConfig,
    target_tables: Vec<String>,
)
{
    let now = get_current_time_for_filename();
    let backup_file = format!("{}_{}.sql", project, &now);
    let backup_path = config.backup_path() + "/" + &backup_file;

    print_sep();
    println!("{} : {}", "Project      ".green(), &config.project());
    println!("{} : {}", "Environment  ".green(), &config.environment());
    println!("{} : {}", "Backup path  ".green(), &backup_path);
    println!("{} : {}", "Database name".green(), &config.db_name());
    if target_tables.len() > 0
    {
        println!("{} : {}", "Target tables".green(), &target_tables.join(", "));
    }
    print_sep();
    println!("Exporting...\n");

    //  Gets the backup file.
    println!("Dumping...\n");
    let session = get_session(project).await;
    let target_tables = target_tables
        .iter()
        .map(|x| x.as_str())
        .collect::<Vec<&str>>();
    let dump = session
        .command("mysqldump")
        .args([
            "-h", &config.db_host_reader(),
            "-P", &config.db_port().to_string(),
            "-u", &config.db_root_user(),
            &("-p".to_string() + &config.db_root_password()),
            &config.db_name(),
        ])
        .args(target_tables)
        .output()
        .await
        .unwrap();
    println!("{}", String::from_utf8_lossy(&dump.stderr));
    let mut file = File::create(&backup_path).unwrap();
    file.write_all(&dump.stdout).unwrap();
    session.close().await.unwrap();
    println!("Done.");
}
