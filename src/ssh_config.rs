//! SSH config structure.

use crate::connect_info::ConnectInfo;

use serde::{ Deserialize, Serialize };

//------------------------------------------------------------------------------
/// Structure that stores operations and connection information for the
/// connection destination.
//------------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct SshConfig
{
    project: String,
    environment: String,
    remote_path: Option<String>,
    remote_cache_path: Option<String>,
    git_path: Option<String>,
    git_src_path: Option<String>,
    backup_path: Option<String>,
    db_host: Option<String>,
    db_host_reader: Option<String>,
    db_port: Option<u16>,
    db_name: Option<String>,
    db_user: Option<String>,
    db_password: Option<String>,
    db_root_user: Option<String>,
    db_root_password: Option<String>,
    connect_info: ConnectInfo,
    tunnels: Option<Vec<ConnectInfo>>,
}

impl SshConfig
{
    //--------------------------------------------------------------------------
    /// Returns the project name.
    //--------------------------------------------------------------------------
    pub fn project( &self ) -> String
    {
        self.project.clone()
    }

    //--------------------------------------------------------------------------
    /// Returns the environment name.
    //--------------------------------------------------------------------------
    pub fn environment( &self ) -> String
    {
        self.environment.clone()
    }

    //--------------------------------------------------------------------------
    /// Returns the remote path.
    //--------------------------------------------------------------------------
    pub fn remote_path( &self ) -> String
    {
        self.remote_path
            .clone()
            .unwrap_or("".to_string())
            .trim_end_matches("/")
            .to_string()
    }

    //--------------------------------------------------------------------------
    /// Returns the remote cache path.
    //--------------------------------------------------------------------------
    pub fn remote_cache_path( &self ) -> String
    {
        self.remote_cache_path
            .clone()
            .unwrap_or("".to_string())
            .trim_end_matches("/")
            .to_string()
    }

    //--------------------------------------------------------------------------
    /// Returns the git path.
    //--------------------------------------------------------------------------
    pub fn git_path( &self ) -> String
    {
        self.git_path
            .clone()
            .unwrap_or("".to_string())
            .trim_end_matches("/")
            .to_string()
    }

    //--------------------------------------------------------------------------
    /// Returns the git source path.
    //--------------------------------------------------------------------------
    pub fn git_src_path( &self ) -> String
    {
        self.git_src_path
            .clone()
            .unwrap_or("".to_string())
            .trim_end_matches("/")
            .to_string()
    }

    //--------------------------------------------------------------------------
    /// Get relative path from Git base path to Git source path.
    //--------------------------------------------------------------------------
    pub fn get_git_relative_path( &self ) -> String
    {
        let git_path = self.git_path();
        let git_src_path = self.git_src_path();

        git_src_path
            .replace(&git_path, "")
            .trim_start_matches("/")
            .to_string()
    }

    //--------------------------------------------------------------------------
    /// Returns the backup path.
    //--------------------------------------------------------------------------
    pub fn backup_path( &self ) -> String
    {
        self.backup_path
            .clone()
            .unwrap_or("".to_string())
            .trim_end_matches("/")
            .to_string()
    }

    //--------------------------------------------------------------------------
    /// Returns the database host.
    //--------------------------------------------------------------------------
    pub fn db_host( &self ) -> String
    {
        self.db_host.clone().unwrap_or("".to_string())
    }

    //--------------------------------------------------------------------------
    /// Returns the database host for reader.
    //--------------------------------------------------------------------------
    pub fn db_host_reader( &self ) -> String
    {
        self.db_host_reader.clone().unwrap_or(self.db_host())
    }

    //--------------------------------------------------------------------------
    /// Returns the database port.
    //--------------------------------------------------------------------------
    pub fn db_port( &self ) -> u16
    {
        self.db_port.unwrap_or(3306)
    }

    //--------------------------------------------------------------------------
    /// Returns the database name.
    //--------------------------------------------------------------------------
    pub fn db_name( &self ) -> String
    {
        self.db_name.clone().unwrap_or("".to_string())
    }

    //--------------------------------------------------------------------------
    /// Returns the database user.
    //--------------------------------------------------------------------------
    pub fn db_user( &self ) -> String
    {
        self.db_user.clone().unwrap_or("root".to_string())
    }

    //--------------------------------------------------------------------------
    /// Returns the database password.
    //--------------------------------------------------------------------------
    pub fn db_password( &self ) -> String
    {
        self.db_password.clone().unwrap_or("".to_string())
    }

    //--------------------------------------------------------------------------
    /// Returns the database root user.
    //--------------------------------------------------------------------------
    pub fn db_root_user( &self ) -> String
    {
        self.db_root_user.clone().unwrap_or("root".to_string())
    }

    //--------------------------------------------------------------------------
    /// Returns the database root password.
    //--------------------------------------------------------------------------
    pub fn db_root_password( &self ) -> String
    {
        self.db_root_password.clone().unwrap_or("".to_string())
    }

    //--------------------------------------------------------------------------
    /// Returns the connect info.
    //--------------------------------------------------------------------------
    pub fn connect_info( &self ) -> &ConnectInfo
    {
        &self.connect_info
    }

    //--------------------------------------------------------------------------
    /// Returns the tunnels.
    //--------------------------------------------------------------------------
    pub fn tunnels( &self ) -> &Option<Vec<ConnectInfo>>
    {
        &self.tunnels
    }
}
