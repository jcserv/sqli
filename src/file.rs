use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::collection::Collection;

/// Get the user's config directory for the application
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("sqli");

    fs::create_dir_all(&config_dir)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700))?;
    }

    Ok(config_dir)
}

/// Read a file as a string
pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

/// Write a string to a file with proper permissions
pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?
            .write_all(content.as_bytes())?;
        
        return Ok(());
    }

    #[cfg(not(unix))]
    {
        fs::write(path, content)?;
        Ok(())
    }
}

pub fn load_collections_from_dir(dir: &Path, collections: &mut Vec<Collection>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            let mut files = Vec::new();
            for file_entry in fs::read_dir(&path)? {
                let file_entry = file_entry?;
                let file_path = file_entry.path();
                
                if file_path.is_file() && file_path.extension().and_then(|s| s.to_str()) == Some("sql") {
                    if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
                        files.push(file_name.to_string());
                    }
                }
            }
            
            collections.push(Collection { name, files });
        }
    }
    
    Ok(())
}

pub fn load_sql_content(collection_name: &str, file_name: &str) -> Result<String> {
    let local_path = PathBuf::from("./sqli")
        .join(collection_name)
        .join(file_name);
    
    if local_path.exists() {
        return read_file_to_string(local_path);
    }
    
    if let Some(config_dir) = dirs::config_dir() {
        let user_path = config_dir
            .join("sqli")
            .join("collections")
            .join(collection_name)
            .join(file_name);
        
        if user_path.exists() {
            return read_file_to_string(user_path);
        }
    }
    
    anyhow::bail!("SQL file not found: {}/{}", collection_name, file_name)
}

pub fn load_yaml_config<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    if !path.exists() {
        return Err(anyhow::anyhow!("Config file not found: {:?}", path));
    }

    let contents = read_file_to_string(path)?;
    let config: T = serde_yaml::from_str(&contents)?;
    Ok(config)
}

pub fn save_yaml_config<T: Serialize>(path: &Path, config: &T) -> Result<()> {
    let yaml = serde_yaml::to_string(config)?;
    write_file(path, &yaml)?;
    Ok(())
}