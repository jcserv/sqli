use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{collection::CollectionScope, file::FileSystem, sql::interface::{get_sql_type, SQLType}};

pub const CONFIG_FILE_NAME: &str = "config.yaml";

#[allow(clippy::too_many_arguments)]
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
pub struct ConfigFile {
    pub connections: Vec<Connection>,
}

pub struct Config {
    connections_map: BTreeMap<String, Connection>, 
    original_names: BTreeMap<String, String>,
}

impl Config {
    fn from_file(config_file: ConfigFile) -> Self {
        let mut connections_map = BTreeMap::new();
        let mut original_names = BTreeMap::new();
        
        for conn in config_file.connections {
            let lowercase_name = conn.name.to_lowercase();
            original_names.insert(lowercase_name.clone(), conn.name.clone());
            connections_map.insert(lowercase_name, conn);
        }
        
        Self { connections_map, original_names }
    }
    
    fn to_file(&self) -> ConfigFile {
        let connections = self.connections_map
            .values()
            .cloned()
            .collect();
        
        ConfigFile { connections }
    }
}

impl Connection {
    pub fn to_url(&self, password: Option<String>) -> String {
        let pwd = password.or(self.password.clone());
        
        match pwd {
            Some(pass) => format!(
                "{}://{}:{}@{}:{}/{}",
                self.conn, self.user, pass, self.host, self.port, self.database
            ),
            None => format!(
                "{}://{}@{}:{}/{}",
                self.conn, self.user, self.host, self.port, self.database
            ),
        }
    }

    pub fn requires_password(&self) -> bool {
        self.password.is_none()
    }
}

pub struct ConfigManager {
    fs: FileSystem,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            fs: FileSystem::new()?,
        })
    }

    pub fn with_filesystem(fs: FileSystem) -> Self {
        Self { fs }
    }

    pub fn load_config(&self) -> Result<Config> {
        let config_path = self.fs.get_scoped_path(CollectionScope::User, CONFIG_FILE_NAME)?;
        if !config_path.exists() {
            return Ok(Config { 
                connections_map: BTreeMap::new(),
                original_names: BTreeMap::new(),
            });
        }
    
        let config_file: ConfigFile = self.fs.load_yaml_config(&config_path)?;
        Ok(Config::from_file(config_file))
    }

    pub fn save_config(&self, config: &Config) -> Result<()> {
        let config_path = self.fs.get_scoped_path(CollectionScope::User, CONFIG_FILE_NAME)?;
        let config_file = config.to_file();
        self.fs.save_yaml_config(&config_path, &config_file)
    }

    pub fn add_connection(&mut self, connection: Connection) -> Result<()> {
        let mut config = self.load_config()?;
        
        let lowercase_name = connection.name.to_lowercase();
        
        if config.connections_map.contains_key(&lowercase_name) {
            println!("Note: Updated existing connection '{}' (names are case-insensitive)", 
                    connection.name);
        }
        
        config.original_names.insert(lowercase_name.clone(), connection.name.clone());
        config.connections_map.insert(lowercase_name, connection);
        
        self.save_config(&config)?;
        Ok(())
    }

    pub fn get_connection(&self, name: &str) -> Result<Option<Connection>> {
        let config = self.load_config()?;
        Ok(config.connections_map.get(&name.to_lowercase()).cloned())
    }

    pub fn list_connections(&self) -> Result<Vec<String>> {
        let config = self.load_config()?;
        Ok(config.original_names.values().cloned().collect())
    }
}