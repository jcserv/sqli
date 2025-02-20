use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub async fn run_config_add(config_manager: &mut ConfigManager, name: String, conn_type: String, host: String, port: u16, database: String, user: String, password: Option<String>) -> Result<()> {
    let connection = Connection {
        name,
        conn_type,
        host,
        port,
        database,
        user,
        password,
        server_ca: None,
        client_cert: None,
        client_key: None,
    };
    config_manager.add_connection(connection)?;
    println!("✅ Connection added successfully");
    Ok(())
}

pub async fn run_config_list(config_manager: &mut ConfigManager) -> Result<()> {
    let connections = config_manager.list_connections()?;

    if connections.is_empty() {
        println!("No connections configured. Try running `sqli config add` to add a new connection.");
        return Ok(());
    }

    println!("Configured connections:");
    for conn in connections {
        println!("  - {}", conn);
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connection {
    pub name: String,
    pub conn_type: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    
    // Optional SSL configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_ca: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_cert: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub connections: Vec<Connection>,
}

impl Connection {
    pub fn to_url(&self) -> String {
        let password = match &self.password {
            Some(pwd) => pwd.clone(),
            None => rpassword::prompt_password("Enter database password: ")
                .expect("Failed to read password"),
        };

        format!(
            "{}://{}:{}@{}:{}/{}",
            self.conn_type, self.user, password, self.host, self.port, self.database
        )
    }
}

pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("sqli");

        fs::create_dir_all(&config_dir)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700))?;
        }

        Ok(Self { config_dir })
    }

    pub fn load_config(&self) -> Result<Config> {
        let config_path = self.config_dir.join("connections.yaml");
        
        if !config_path.exists() {
            return Ok(Config { connections: Vec::new() });
        }

        let contents = fs::read_to_string(config_path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_config(&self, config: &Config) -> Result<()> {
        let config_path = self.config_dir.join("connections.yaml");
        
        let config_to_save = Config {
            connections: config.connections.iter().map(|conn| {
                    conn.clone()
            }).collect(),
        };

        let yaml = serde_yaml::to_string(&config_to_save)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&config_path)?
                .write_all(yaml.as_bytes())?;
        }

        Ok(())
    }

    pub fn add_connection(&mut self, connection: Connection) -> Result<()> {
        let mut config = self.load_config()?;
        
        // Remove any existing connection with the same name
        config.connections.retain(|c| c.name != connection.name);
        
        config.connections.push(connection);
        self.save_config(&config)?;
        
        Ok(())
    }

    pub fn get_connection(&self, name: &str) -> Result<Option<Connection>> {
        let config = self.load_config()?;
        Ok(config.connections.into_iter().find(|c| c.name == name))
    }

    pub fn list_connections(&self) -> Result<Vec<String>> {
        let config = self.load_config()?;
        Ok(config.connections.into_iter().map(|c| c.name).collect())
    }
}