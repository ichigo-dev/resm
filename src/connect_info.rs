//! Information required for SSH connection.

use serde::{ Deserialize, Serialize };

//------------------------------------------------------------------------------
/// A structure that summarizes information necessary for SSH connection.
//------------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectInfo
{
    host: String,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    identity_file: Option<String>,
}

impl ConnectInfo
{
    //--------------------------------------------------------------------------
    /// Returns the host.
    //--------------------------------------------------------------------------
    pub fn host( &self ) -> String
    {
        self.host.clone()
    }

    //--------------------------------------------------------------------------
    /// Returns the port.
    //--------------------------------------------------------------------------
    pub fn port( &self ) -> u16
    {
        self.port.unwrap_or(22)
    }

    //--------------------------------------------------------------------------
    /// Returns the user.
    //--------------------------------------------------------------------------
    pub fn user( &self ) -> String
    {
        self.user.clone().unwrap_or("".to_string())
    }

    //--------------------------------------------------------------------------
    /// Returns the password.
    //--------------------------------------------------------------------------
    pub fn password( &self ) -> String
    {
        self.password.clone().unwrap_or("".to_string())
    }

    //--------------------------------------------------------------------------
    /// Returns the path to identity file.
    //--------------------------------------------------------------------------
    pub fn identity_file( &self ) -> String
    {
        self.identity_file
            .clone()
            .unwrap_or("".to_string())
            .trim_end_matches("/")
            .to_string()
    }
}
