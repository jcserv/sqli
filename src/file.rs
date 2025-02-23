use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::collection::{Collection, CollectionScope, SelectedFile};

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

pub fn load_collections_from_dir(dir: &Path, collections: &mut Vec<Collection>, scope: CollectionScope) -> Result<()> {
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
            
            collections.push(Collection { name, files, scope: scope.clone() });
        }
    }
    
    Ok(())
}

pub fn load_config_content(scope: CollectionScope) -> Result<String> {
    read_scoped_file(scope, "config.yaml")
}

pub fn load_sql_with_scope(collection_name: &str, file_name: &str, scope: CollectionScope) -> Result<String> {
    let relative_path = PathBuf::from(collection_name).join(file_name);
    let full_path = get_scoped_path(scope, relative_path)?;
    
    if !full_path.exists() {
        return Err(anyhow::anyhow!(
            "SQL file not found: {} (scope: {:?})", 
            full_path.display(), 
            scope,
        ));
    }
    
    read_file_to_string(&full_path)
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

pub fn get_scoped_path(scope: CollectionScope, relative_path: impl AsRef<Path>) -> Result<PathBuf> {
    let base_path = match scope {
        CollectionScope::User => dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("sqli"),
        CollectionScope::Cwd => PathBuf::from("./sqli"),
    };
    
    let relative_path = relative_path.as_ref();
    if relative_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(anyhow::anyhow!("Invalid path: contains parent directory references"));
    }
    
    Ok(base_path.join(relative_path))
}

pub fn read_scoped_file(scope: CollectionScope, relative_path: impl AsRef<Path>) -> Result<String> {
    let path = get_scoped_path(scope, relative_path)?;
    if path.exists() {
        read_file_to_string(&path)
    } else {
        Err(anyhow::anyhow!("File not found: {:?}", path))
    }
}

pub fn save_file(selected_file: &SelectedFile, content: &str) -> Result<()> {
    match selected_file {
        SelectedFile::Config(scope) => {
            let config_path = get_scoped_path(*scope, "config.yaml")?;
            write_file(config_path, content)?;
            Ok(())
        },
        SelectedFile::Sql { collection, filename, scope } => {
            let relative_path = PathBuf::from(collection).join(filename);
            let full_path = get_scoped_path(*scope, relative_path)?;
            write_file(full_path, content)?;
            Ok(())
        }
    }
}

pub fn parse_selected_file(selected: &[String]) -> Option<SelectedFile> {
    let file_item = selected.last()?;
    
    let scope = if let Some(collection_name) = selected.get(selected.len().saturating_sub(2)) {
        if collection_name.contains("(user)") {
            CollectionScope::User
        } else {
            CollectionScope::Cwd
        }
    } else {
        if file_item.contains("(user)") {
            CollectionScope::User
        } else {
            CollectionScope::Cwd
        }
    };

    if file_item.contains("config.yaml") {
        Some(SelectedFile::Config(scope))
    } else if file_item.ends_with(".sql") {
        let collection = selected.get(selected.len().saturating_sub(2))
            .and_then(|s| s.split(" (").next())
            .map(String::from)?;
        
        Some(SelectedFile::Sql {
            collection,
            filename: file_item.to_string(),
            scope
        })
    } else {
        None
    }
}