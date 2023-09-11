//! Utility commands to automate these tasks from a JSON file that summarizes
//! settings related to SSH connections to remote servers, application
//! deployment, etc.
//! 
//! # JSON format
//! 
//! ```json
//! [
//!     {
//!         "project": "project name",
//!         "environment": "dev",
//! 
//!         "remote_path": "path/to/remote/project/root",
//!         "remote_cache_path": "path/to/remote/project/cache",
//! 
//!         "git_path": "path/to/git/repo",
//!         "git_src_path": "path/to/git/src",
//!         "backup_path": "path/to/backup",
//! 
//!         "db_host": "db host name",
//!         "db_host_reader": "db host name for reader",
//!         "db_port": 3306,
//!         "db_name": "db name",
//!         "db_user": "db user",
//!         "db_password": "db password",
//!         "db_root_user": "db root user",
//!         "db_root_password": "db root password",
//! 
//!         "connect_info":
//!         {
//!             "host": "host name",
//!             "port": 22,
//!             "user": "user",
//!             "identity_file": "path/to/key_file"
//!         },
//! 
//!         "tunnels":
//!         [
//!             {
//!                 "host": "bastion host name",
//!                 "port": 22,
//!                 "user": "user",
//!                 "identity_file": "path/to/key_file"
//!             }
//!         ]
//!     }
//! ]
//! ```
//! 
//! ## Fields
//! 
//! Required fields are marked with an asterisk (*).
//! 
//! - project (*): Optional project name
//! - environment (*): Optional environment name
//! - remote_path: Reference path to be operated in the destination server
//!                (absolute path)
//! - remote_cache_path: Path to the cache directory in the destination server
//!                      (absolute path)
//! - git_path: Path to the git repository (absolute path)
//! - git_src_path: Path to the source directory in the git repository
//!                 (absolute path)
//! - backup_path: Path to the backup directory (absolute path)
//! - db_host: Host name of the database server
//! - db_host_reader: Host name of the database server for reader
//! - db_port: Port number of the database server
//! - db_name: Database name
//! - db_user: Database user name
//! - db_password: Database password
//! - db_root_user: Database root user name
//! - db_root_password: Database root password
//! - connect_info (*)
//!     - host (*): Host name or IP address to connect to
//!     - port: Port number of the server
//!     - user: User name
//!     - password: Password (Entering password cannot be omitted)
//!     - identity_file: Path to the identity file (absolute path)
//! - tunnels: Information on the step server to be passed through when
//!            connecting (array of connect_info)
//! 
//! 
//! # Commands
//! 
//! ## init
//! 
//! Generates SSH config file from the JSON file.
//! 
//! ## list
//! 
//! Lists projects.
//! 
//! ## show
//! 
//! Shows the project setting.
//! 
//! ## replace
//! 
//! Replaces the project directory in the destination server with the local
//! project directory. In short, it is useful when you want to perform a full
//! update of an application.
//! 
//! ## patch
//! 
//! Uploads only specified files in the local repository to the remote server.
//! 
//! ## clear
//! 
//! Clears the remote cache directory.
//! 
//! ## backup
//! 
//! Backs up the remote directory.
//! 
//! ## backup-db
//! 
//! Backs up the database.

#![allow(dead_code)]

mod connect_info;
mod ssh_config;
mod generate;
mod upload;
mod backup;
mod util;

use generate::generate_ssh_config;
use upload::{ upload_all, upload_patch, clear_cache };
use backup::{ backup, backup_db };
use util::{ load_json, get_session };

use std::env;

use clap::{ Parser, Subcommand };

//------------------------------------------------------------------------------
/// Parsed command line arguments.
//------------------------------------------------------------------------------
#[derive(Debug, Parser)]
#[command(
    name = "RESM - Remote Server Management tools",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
)]
struct Cli
{
    #[command(subcommand)]
    subcommand: Subcommands,

    /// Path to the JSON file that summarizes the settings related to SSH
    /// connections to remote servers, application deployment, etc.
    #[clap(
        short = 'e',
        long,
        default_value = "",
    )]
    env_path: String,
}

//------------------------------------------------------------------------------
/// Subcommands.
//------------------------------------------------------------------------------
#[derive(Debug, Subcommand)]
enum Subcommands
{
    /// Generate SSH config file from the JSON file.
    Init,

    /// List projects.
    List,

    /// Show the project setting.
    Show
    {
        /// Project name.
        #[clap(
            required = true,
        )]
        project: String,
    },

    /// Replace the project directory in the destination server with the local
    /// project directory.
    Replace
    {
        /// Project name.
        #[clap(
            required = true,
        )]
        project: String,

        /// Relative path from the project directory that you want to upload.
        #[clap(
            short = 't',
            long,
            default_value = "",
        )]
        target_path: String,

        /// Whether to compress files when uploading.
        #[clap(
            long,
        )]
        zip: bool,
    },

    /// Upload only specified files in the local repository to the remote
    /// server.
    Patch
    {
        /// Project name.
        #[clap(
            required = true,
        )]
        project: String,

        /// Path to the file that describes the files to be uploaded.
        #[clap(
            short = 'f',
            long,
            default_value = "",
        )]
        patch_file: String,
    },

    /// Clear the remote cache directory.
    Clear
    {
        /// Project name.
        #[clap(
            required = true,
        )]
        project: String,
    },

    /// Back up the remote directory.
    Backup
    {
        /// Project name.
        #[clap(
            required = true,
        )]
        project: String,

        ///  Relative path from the project directory that you want to backup.
        #[clap(
            short = 't',
            long,
            default_value = "",
        )]
        target_path: String,
    },

    /// Back up the database.
    BackupDb
    {
        /// Project name.
        #[clap(
            required = true,
        )]
        project: String,

        /// Specify table names to be backed up separated by commas (be careful
        /// not to include spaces, etc.)
        #[clap(
            short = 't',
            long,
            value_parser,
            num_args = 1..,
            value_delimiter = ',',
        )]
        target_tables: Vec<String>,
    },
}

#[tokio::main]
async fn main()
{
    let cli = Cli::parse();

    //  Loads JSON file.
    let env_path = if cli.env_path.len() > 0
    {
        cli.env_path
    }
    else
    {
        env::var("HOME").unwrap_or("".to_string()) + "/env"
    };
    let config_entries = load_json(&env_path);

    //  Executes subcommand.
    match cli.subcommand
    {
        Subcommands::Init => generate_ssh_config(&config_entries),
        Subcommands::List =>
        {
            let keys = config_entries.keys();
            for key in keys
            {
                println!("{}", key);
            }
        },
        Subcommands::Show { project } =>
        {
            if let Some(config) = config_entries.get(&project)
            {
                println!("{}", serde_json::to_string_pretty(&config).unwrap());
            }
            else
            {
                println!("Project not found.");
                return;
            }
        },
        Subcommands::Replace { project, target_path, zip } =>
        {
            if let Some(config) = config_entries.get(&project)
            {
                upload_all(&project, config, target_path, zip).await;
            }
            else
            {
                println!("Project not found.");
                return;
            }
        },
        Subcommands::Patch { project, patch_file } =>
        {
            if let Some(config) = config_entries.get(&project)
            {
                upload_patch(&project, config, patch_file).await;
            }
            else
            {
                println!("Project not found.");
                return;
            }
        },
        Subcommands::Clear { project } =>
        {
            if let Some(config) = config_entries.get(&project)
            {
                let session = get_session(&project).await;
                clear_cache(&session, &config.remote_cache_path()).await;
                session.close().await.unwrap();
            }
            else
            {
                println!("Project not found.");
                return;
            }
        }
        Subcommands::Backup { project, target_path } =>
        {
            if let Some(config) = config_entries.get(&project)
            {
                backup(&project, config, target_path).await;
            }
            else
            {
                println!("Project not found.");
                return;
            }
        },
        Subcommands::BackupDb { project, target_tables } =>
        {
            if let Some(config) = config_entries.get(&project)
            {
                backup_db(&project, config, target_tables).await;
            }
            else
            {
                println!("Project not found.");
                return;
            }
        },
    }
}
