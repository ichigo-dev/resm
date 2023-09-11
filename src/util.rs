//! Utility functions.

use crate::ssh_config::SshConfig;

use std::collections::BTreeMap;
use std::path::PathBuf;

use glob::glob;
use colored::Colorize;
use chrono::Local;
use ssh::{ Session, Channel };

//------------------------------------------------------------------------------
/// Loads JSON files.
//------------------------------------------------------------------------------
pub fn load_json( path: &str ) -> BTreeMap<String, SshConfig>
{
    //  Gets the path to JSON files.
    let json_path = path.trim_end_matches("/").to_string() + "/**/*.json";

    //  Loads JSON files.
    let mut config_entries: Vec<SshConfig> = Vec::new();
    for entry in glob(&json_path).unwrap()
    {
        match entry
        {
            Ok(path) =>
            {
                let json_data = std::fs::read_to_string(path).unwrap();
                match serde_json::from_str::<Vec<SshConfig>>(&json_data)
                {
                    Ok(config_entry) => config_entries.extend(config_entry),
                    Err(e) => println!("{:?}", e),
                }
            },
            Err(e) => println!("{:?}", e),
        }
    }

    config_entries
        .into_iter()
        .map(|entry|
        {
            let mut key = entry.project();
            if entry.environment().len() > 0
            {
                key += &("_".to_string() + entry.environment().as_str());
            }
            (key, entry)
        })
        .collect()
}

//------------------------------------------------------------------------------
/// Gets SSH channel for the specified project.
//------------------------------------------------------------------------------
pub fn get_session( project: &str ) -> Session
{
    let mut session = Session::new().unwrap();
    session.set_host(project).unwrap();
    session.parse_config(None).unwrap();
    session.connect().unwrap();
    session.userauth_publickey_auto(None).unwrap();
    session.channel_new().unwrap();
    session
}

//------------------------------------------------------------------------------
/// Waits for the specified channel to finish.
//------------------------------------------------------------------------------
pub fn wait_exec( channel: &Channel )
{
    while channel.get_exit_status().is_none()
    {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

//------------------------------------------------------------------------------
/// Gets file paths in the specified directory.
//------------------------------------------------------------------------------
pub fn get_file_paths( dir: &str ) -> Vec<PathBuf>
{
    let mut file_paths: Vec<PathBuf> = Vec::new();
    for entry in glob(&(dir.to_string() + "/**/*")).unwrap()
    {
        match entry
        {
            Ok(path) =>
            {
                if path.is_dir()
                {
                    continue;
                }
                file_paths.push(path);
            },
            Err(e) => println!("{:?}", e),
        }
    }
    file_paths
}

//------------------------------------------------------------------------------
/// Prints separator.
//------------------------------------------------------------------------------
pub fn print_sep()
{
    println!("{}", "=".to_string().repeat(80));
}

//------------------------------------------------------------------------------
/// Confirm.
//------------------------------------------------------------------------------
pub fn confirm( message: &str ) -> bool
{
    println!("{} [y/N]", message.yellow());
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim() == "y"
}

//------------------------------------------------------------------------------
/// Gets current time for filename.
//------------------------------------------------------------------------------
pub fn get_current_time_for_filename() -> String
{
    let now = Local::now();
    now.format("%Y%m%d_%H%M%S").to_string()
}
