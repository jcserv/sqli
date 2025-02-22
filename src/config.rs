use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{file, sql::interface::{get_sql_type, SQLType}};

pub async fn run_config_set(config_manager: &mut ConfigManager, name: String, conn: String, 
    host: String, port: u16, database: String, user: String, password: Option<String>) -> Result<()> {
    let c = get_sql_type(&conn).ok_or_else(|| {
        anyhow::anyhow!("Unsupported SQL type: '{}'. Supported types: postgresql", conn)
    })?;

    let connection = Connection {
        name,
        conn: c,
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
        println!("No connections configured. Try running `sqli config set` to configure a new connection.");
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
    pub conn: SQLType,
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
            None => return "".to_string(),
        };

        format!(
            "{}://{}:{}@{}:{}/{}",
            self.conn, self.user, password, self.host, self.port, self.database
        )
    }
}

pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = file::get_config_dir()?;
        Ok(Self { config_dir })
    }

    pub fn load_config(&self) -> Result<Config> {
        let config_path = self.config_dir.join("connections.yaml");
        if !config_path.exists() {
            return Ok(Config { connections: Vec::new() });
        }

        file::load_yaml_config::<Config>(&config_path)
    }

    pub fn save_config(&self, config: &Config) -> Result<()> {
        let config_path = self.config_dir.join("connections.yaml");
        
        let config_to_save = Config {
            connections: config.connections.iter().map(|conn| {
                    conn.clone()
            }).collect(),
        };

        file::save_yaml_config(&config_path, &config_to_save)
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