//! What the gen command actually does.

use crate::connect_info::ConnectInfo;
use crate::ssh_config::SshConfig;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;

use colored::Colorize;

//------------------------------------------------------------------------------
/// Generates SSH configuration file from SshConfig map.
///
/// # Arguments
///
/// - `config_entries` - SshConfig map.
//------------------------------------------------------------------------------
pub fn generate_ssh_config( config_entries: &BTreeMap<String, SshConfig> )
{
    println!("Generating SSH config file.");
    let mut ssh_config = String::new();
    for (project_name, entry) in config_entries
    {
        //  For generating ProxyJump.
        let mut jump_host = None;
        let mut config_block = String::new();

        //  Adds tunnels.
        if let Some(tunnels) = entry.tunnels()
        {
            for (i, tunnel) in tunnels.iter().enumerate()
            {
                let mut name = format!("{}_bastion", project_name);
                if tunnels.len() > 1
                {
                    name += &format!("_{}", i);
                }
                let config = generate_ssh_config_item
                (
                    &name,
                    tunnel,
                    jump_host,
                    false,
                );
                config_block = config + &config_block;
                jump_host = Some(name);
            }
        }

        //  Adds host.
        let config = generate_ssh_config_item
        (
            project_name,
            entry.connect_info(),
            jump_host.clone(),
            true,
        );

        ssh_config += &(config + &config_block);

        if entry.db_host().len() > 0
        {
            let name = format!("{}_db", project_name);
            let mut config = generate_ssh_config_item
            (
                &name,
                entry.connect_info(),
                jump_host.clone(),
                true,
            );
            config += &format!
            (
                "    RemoteCommand mysql -h {} -P {} -u {} -p{} {}\n",
                entry.db_host(),
                entry.db_port(),
                entry.db_user(),
                entry.db_password(),
                entry.db_name(),
            );
            config += "    RequestTTY yes\n";
            ssh_config += &config;
        }
        if entry.db_host() != entry.db_host_reader()
        {
            let name = format!("{}_db_reader", project_name);
            let mut config = generate_ssh_config_item
            (
                &name,
                entry.connect_info(),
                jump_host.clone(),
                true,
            );
            config += &format!
            (
                "    RemoteCommand mysql -h {} -P {} -u {} -p{} {}\n",
                entry.db_host_reader(),
                entry.db_port(),
                entry.db_user(),
                entry.db_password(),
                entry.db_name(),
            );
            config += "    RequestTTY yes\n";
            ssh_config += &config;
        }
    }
    let mut config_file = File::create("config").unwrap();
    config_file.write_all(ssh_config.as_bytes()).unwrap();
    println!("Done.\n");
    println!
    (
        "{}\n      {}",
        "Hint: Copy the generated config file to ~/.ssh/config.".yellow(),
        "(if it is already in ~/.ssh/config, add it appropriately)".yellow()
    );
}

//------------------------------------------------------------------------------
/// Generates SSH configuration item.
///
/// # Arguments
///
/// - `name` - Project name.
/// - `connect_info` - ConnectInfo.
/// - `jump_host` - Jump host name.
/// - `comment` - Whether to add a comment.
//------------------------------------------------------------------------------
fn generate_ssh_config_item
(
    name: &str,
    connect_info: &ConnectInfo,
    jump_host: Option<String>,
    comment: bool,
) -> String
{
    let host_name = connect_info.host();
    let port = connect_info.port();
    let user = connect_info.user();
    let identity_file = connect_info.identity_file();
    let mut config = String::new();

    //  Adds comment.
    if comment
    {
        config += &format!(
            r#"
#==========================================================
# {}
#=========================================================="#,
            name,
        );
    }

    //  Adds host.
    config += &format!
    (
        r#"
Host {}
    HostName {}
    Port {}
    User {}
    IdentityFile {}
"#,
        name,
        host_name,
        port,
        user,
        identity_file,
    );

    //  Adds ProxyCommand.
    if let Some(jump_host) = jump_host
    {
        config += &format!
        (
            "    ProxyJump {}\n",
            jump_host,
        );
    }

    config
}
