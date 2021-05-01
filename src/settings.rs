use std::net::{SocketAddr, ToSocketAddrs};
use std::{env, io, vec};

use config::{Config, ConfigError, Environment, File};
use directories::ProjectDirs;
use serde::{Serialize, Deserialize};
use std::ops::Deref;

/// Default server name
pub const DEFAULT_SERVER_NAME: &'static str = "default";
pub const DEFAULT_SERVER_HOST: &'static str = "localhost";


/// Default server port number for SMTP protocol
pub const DEFAULT_SMTP_PORT: u16 = 25;

/// Default server port number for POP3 protocol
pub const DEFAULT_POP3_PORT: u16 = 110;

/// Default server port number for IMAP protocol
pub const DEFAULT_IMAP_PORT: u16 = 143;

/// Default server port number for SMTP protocol over secure (TLS) channel
pub const DEFAULT_SMTP_TLS_PORT: u16 = 465;

/// Default server port number for POP3 protocol over secure (TLS) channel
pub const DEFAULT_POP3_TLS_PORT: u16 = 995;

/// Default server port number for IMAP protocol over secure (TLS) channel
pub const DEFAULT_IMAP_TLS_PORT: u16 = 993;

/// Application settings configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    servers: Vec<Server>,
}

/// Configuration parameters of a server
#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    name: String,
    smtp: Smtp,
    imap: Imap,
    credentials: Credentials,
}

/// TLS configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Tls {
    port: u16,
}

/// Configuration of an SMTP server connection settings
#[derive(Debug, Serialize, Deserialize)]
pub struct Smtp {
    host: String,
    port: u16,
    tls: Option<Tls>,
}

/// Configuration of an IMAP server connection settings
#[derive(Debug, Serialize, Deserialize)]
pub struct Imap {
    host: String,
    port: u16,
    tls: Option<Tls>,
}

/// Configuration of an IMAP server connection settings
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Credentials {
    None,
    UsernameAndPassword{
        username: String,
        password: String,
    },
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let mut cfg = Config::new();

        let project_dirs = ProjectDirs::from("org", "postkast", "Postkast").ok_or(
            ConfigError::Message("Cannot locate project directories".to_string()),
        )?;

        let config_file = project_dirs.preference_dir().join("Settings.toml");
        println!("Loading settings from {:?}", &config_file);
        cfg.merge(File::from(config_file).required(false))?;

        cfg.merge(Environment::with_prefix("POSTKAST_"))?;

        cfg.try_into()
    }

    pub fn print_default() -> Result<(), ConfigError> {
        let mut default_server = Server::default();
        default_server.with_name("GMail")
            .with_imap_host_and_tls_port("imap.google.com", 993)
            .with_username_and_password("username", "password");
        let default_server = default_server;
        let default_settings = Settings { servers: vec![ default_server ]};

        //let project_dirs = ProjectDirs::from("org", "postkast", "Postkast").ok_or(
        //    ConfigError::Message("Cannot locate project directories".to_string()),
        //)?;

        //let preferences_dir = project_dirs.preference_dir();
        //let path = preferences_dir.join("Settings.toml");
        //println!("Storing default prefernces to {:?}", &path);

        let value = toml::Value::try_from(&default_settings).map_err(|err|
            ConfigError::Message(format!("Cannot convert default settings to TOML: {:?}", err))
        )?;

        let contents = value.to_string();
        println!("{:}", contents);

        Ok(())
    }
}

impl Settings {
    /// Iterator over all configured server configurations
    pub fn servers(&self) -> impl Iterator<Item = &Server> + '_ {
        self.servers.iter()
    }
}


impl Default for Server {
    fn default() -> Server {
        Server {
            name: DEFAULT_SERVER_NAME.to_string(),
            smtp: Smtp::default(),
            imap: Imap::default(),
            credentials: Credentials::None,
        }
    }
}

impl Server {
    /// server name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// IMAP server configuration settings
    pub fn imap(&self) -> &Imap {
        &self.imap
    }

    /// SMTP server configuration settings
    pub fn smtp(&self) -> &Smtp {
        &self.smtp
    }

    /// Server credentials 
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}

impl Server {
    pub fn with_name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_string();
        self
    }

    pub fn with_imap(&mut self, imap: Imap) -> &mut Self {
        self.imap = imap;
        self
    }

    pub fn with_imap_host(&mut self, host: &str) -> &mut Self {
        self.imap.with_host(host);
        self
    }

    pub fn with_imap_host_and_port(&mut self, host: &str, port: u16) -> &mut Self {
        self.imap.with_host(host).with_port(port);
        self
    }

    pub fn with_imap_host_and_tls_port(&mut self, host: &str, port: u16) -> &mut Self {
        self.imap.with_host(host).with_tls(Tls { port });
        self
    }

    pub fn with_username_and_password(&mut self, username: &str, password: &str) -> &mut Self {
        let username = username.to_string();
        let password = password.to_string();
        self.credentials = Credentials::UsernameAndPassword {username, password};
        self
    }
}

impl Default for Imap {
    fn default() -> Self {
        Imap {
            host: DEFAULT_SERVER_NAME.to_string(),
            port: DEFAULT_IMAP_PORT,
            tls: None,
        }
    }
}

// Public accessors
impl Imap {
    /// Server hostname
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Server port number
    pub fn port(&self) -> u16 {
        if let Some(tls) = self.tls() {
            tls.port
        } else {
            self.port
        }
    }

    /// Server TLS configuration
    pub fn tls(&self) -> Option<&Tls> {
        self.tls.as_ref()
    }
}

/// Public builder methods
impl Imap {
    /// Update server hostname
    pub fn with_host(&mut self, host: &str) -> &mut Self {
        self.host = host.to_string();
        self
    }

    /// Set IMAP server port settings
    pub fn with_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    /// Set TLS connection settings
    pub fn with_tls(&mut self, tls: Tls) -> &mut Self {
        self.tls = Some(tls);
        self
    }
    
    /// Set up with default IMAP connection settings
    pub fn with_default_tls(&mut self) -> &mut Self {
        self.tls = Some(Tls::default_imap());
        self
    }
}

impl Default for Smtp {
    fn default() -> Self {
        Smtp {
            host: "localhost".to_string(),
            port: DEFAULT_SMTP_PORT,
            tls: None,
        }
    }
}

/// Public accessors
impl Smtp {
    /// Host name of the SMTP server
    fn host(&self) -> &str {
        &self.host
    }

    /// Port number of the SMTP server
    fn port(&self) -> u16 {
        if let Some(tls) = &self.tls {
            tls.port
        } else {
            self.port
        }
    }

    fn tls(&self) -> Option<&Tls> {
        self.tls.as_ref()
    }
}

/// Public builder methods
impl Smtp {
    /// Update server hostname
    pub fn with_host(&mut self, host: &str) -> &mut Self {
        self.host = host.to_string();
        self
    }

    /// Set IMAP server port settings
    pub fn with_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    /// Set TLS connection settings
    pub fn with_tls(&mut self, tls: Tls) -> &mut Self {
        self.tls = Some(tls);
        self
    }
    
    /// Set up with default IMAP connection settings
    pub fn with_default_tls(&mut self) -> &mut Self {
        self.tls = Some(Tls::default_smtp());
        self
    }
}


impl ToSocketAddrs for Imap {
    type Iter = vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        let tuple = (self.host(), self.port());
        tuple.to_socket_addrs()
    }
}

impl Default for Credentials {
    fn default() -> Self {
        Credentials::None
    }
}

impl Tls {
    fn default_imap() -> Tls {
        Tls {
            port: DEFAULT_IMAP_TLS_PORT,
        }
    }

    fn default_smtp() -> Tls {
        Tls {
            port: DEFAULT_SMTP_TLS_PORT,
        }
    }
}