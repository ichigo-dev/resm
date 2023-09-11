//! Specific implementation of operations related to application backup.

use crate::ssh_config::SshConfig;
use crate::util::{
    get_session,
    print_sep,
    get_current_time_for_filename,
};

use std::fs::File;
use std::io::{ Write, Read };

use colored::Colorize;
use ssh::READ;

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
pub fn backup
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
        format!("/{}_{}.zip", project, &now)
    };
    let backup_path = backup_path + "/" + &backup_file;

    print_sep();
    println!("{} : {}", "Project    ".green(), &config.project());
    println!("{} : {}", "Environment".green(), &config.environment());
    println!("{} : {}", "Backup path".green(), &backup_path);
    println!("{} : {}", "Remote path".green(), &remote_path);
    print_sep();

    //  Gets the backup file.
    let mut session = get_session(project);
    {
        let mut channel = session.channel_new().unwrap();
        channel.open_session().unwrap();
        let command = format!("zip -r {} {}", &backup_file, &remote_path);
        channel.request_exec(command.as_bytes()).unwrap();
        channel.send_eof().unwrap();
    }

    {
        let mut scp = session.scp_new(READ, &backup_file).unwrap();
        scp.init().unwrap();
        let mut buf:Vec<u8> = Vec::new();
        scp.reader().read_to_end(&mut buf).unwrap();

        let mut file = File::create(&backup_path).unwrap();
        file.write_all(&buf).unwrap();
    }

    {
        let mut channel = session.channel_new().unwrap();
        channel.open_session().unwrap();
        let command = format!("rm {}", &backup_file);
        channel.request_exec(command.as_bytes()).unwrap();
        channel.send_eof().unwrap();
    }
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
pub fn backup_db
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

    let target_tables = target_tables
        .iter()
        .map(|x| x.as_str())
        .collect::<Vec<&str>>();

    //  Gets the backup file.
    println!("Dumping...\n");
    let mut session = get_session(project);
    {
        let mut channel = session.channel_new().unwrap();
        channel.open_session().unwrap();
        let command = format!("mysqldump -h {} -P {} -u {} -p{} {} {}",
            &config.db_host_reader(),
            &config.db_port().to_string(),
            &config.db_root_user(),
            &config.db_root_password(),
            &config.db_name(),
            &target_tables.join(" "),
        );
        channel.request_exec(command.as_bytes()).unwrap();
        channel.send_eof().unwrap();

        let mut buf = Vec::new();
        channel.stdout().read_to_end(&mut buf).unwrap();
        let mut err = Vec::new();
        channel.stderr().read_to_end(&mut err).unwrap();

        println!("{}", String::from_utf8_lossy(&err));
        let mut file = File::create(&backup_path).unwrap();
        file.write_all(&buf).unwrap();
    }
    println!("Done.");
}
