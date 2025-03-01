use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::{collection::{Collection, CollectionScope, SelectedFile}, config::CONFIG_FILE_NAME};

#[derive(Debug, Clone)]
pub struct FileSystem {
    user_dir: PathBuf,
    workspace_dir: PathBuf,
}

impl FileSystem {
    pub fn new() -> Result<Self> {
        let user_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("sqli");

        let workspace_dir = PathBuf::from("./sqli");
        
        let fs = Self {
            user_dir,
            workspace_dir,
        };
        
        fs.ensure_directories()?;
        
        Ok(fs)
    }
    
    pub fn with_paths(user_dir: PathBuf, workspace_dir: PathBuf) -> Result<Self> {
        let fs = Self {
            user_dir,
            workspace_dir,
        };
        
        fs.ensure_directories()?;
        
        Ok(fs)
    }
    
    fn ensure_directories(&self) -> Result<()> {
        // Ensure user directory exists with proper permissions
        fs::create_dir_all(&self.user_dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.user_dir, fs::Permissions::from_mode(0o700))?;
        }
        
        Ok(())
    }

    pub fn get_base_path(&self, scope: CollectionScope) -> &PathBuf {
        match scope {
            CollectionScope::User => &self.user_dir,
            CollectionScope::Cwd => &self.workspace_dir,
        }
    }

    pub fn get_scoped_path(&self, scope: CollectionScope, relative_path: impl AsRef<Path>) -> Result<PathBuf> {
        let base_path = self.get_base_path(scope);
        let relative_path = relative_path.as_ref();
        
        if relative_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(anyhow::anyhow!("Invalid path: contains parent directory references"));
        }
        
        Ok(base_path.join(relative_path))
    }

    pub fn read_file(&self, path: impl AsRef<Path>) -> Result<String> {
        Ok(fs::read_to_string(path)?)
    }

    pub fn write_file(&self, path: impl AsRef<Path>, content: &str) -> Result<()> {
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

    pub fn load_yaml_config<T: for<'de> Deserialize<'de>>(&self, path: impl AsRef<Path>) -> Result<T> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(anyhow::anyhow!("Config file not found: {:?}", path));
        }

        let contents = self.read_file(path)?;
        let config: T = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_yaml_config<T: Serialize>(&self, path: impl AsRef<Path>, config: &T) -> Result<()> {
        let yaml = serde_yaml::to_string(config)?;
        self.write_file(path, &yaml)?;
        Ok(())
    }

    pub fn load_sql(&self, collection_name: &str, file_name: &str, scope: CollectionScope) -> Result<String> {
        let relative_path = PathBuf::from(collection_name).join(file_name);
        let full_path = self.get_scoped_path(scope, relative_path)?;
        
        if !full_path.exists() {
            return Err(anyhow::anyhow!(
                "SQL file not found: {} (scope: {:?})", 
                full_path.display(), 
                scope
            ));
        }
        
        self.read_file(&full_path)
    }

    pub fn load_collections_from_dir(&self, dir: &Path, collections: &mut Vec<Collection>, scope: CollectionScope) -> Result<()> {
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
                
                collections.push(Collection { name, files, scope });
            }
        }
        
        Ok(())
    }

    pub fn create_file_or_folder(&self, name: &str, is_folder: bool, scope: CollectionScope) -> Result<()> {
        let base_path = self.get_base_path(scope);
        let target_path = base_path.join(name);

        if target_path.exists() {
            return Err(anyhow::anyhow!("File or folder already exists"));
        }

        if is_folder {
            fs::create_dir_all(&target_path)?;
        } else {
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            fs::write(&target_path, "")?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = if is_folder { 0o700 } else { 0o600 };
            fs::set_permissions(&target_path, fs::Permissions::from_mode(mode))?;
        }

        Ok(())
    }

    pub fn rename_file_or_folder(
        &self,
        old_name: &str,
        new_name: &str,
        current_scope: CollectionScope,
        new_scope: CollectionScope
    ) -> Result<()> {
        let old_path = self.get_scoped_path(current_scope, old_name)?;
        let new_path = self.get_scoped_path(new_scope, new_name)?;

        if !old_path.exists() {
            return Err(anyhow::anyhow!("Source file or folder does not exist"));
        }

        if new_path.exists() {
            return Err(anyhow::anyhow!("Target file or folder already exists"));
        }

        if let Some(parent) = new_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if current_scope != new_scope {
            if old_path.is_dir() {
                self.copy_dir_recursive(&old_path, &new_path)?;
                fs::remove_dir_all(&old_path)?;
            } else {
                fs::copy(&old_path, &new_path)?;
                fs::remove_file(&old_path)?;
            }
        } else {
            fs::rename(&old_path, &new_path)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = if new_path.is_dir() { 0o700 } else { 0o600 };
            fs::set_permissions(&new_path, fs::Permissions::from_mode(mode))?;
        }

        Ok(())
    }

    pub fn save_file(&self, selected_file: &SelectedFile, content: &str) -> Result<()> {
        match selected_file {
            SelectedFile::Config(scope) => {
                let config_path = self.get_scoped_path(*scope, CONFIG_FILE_NAME)?;
                self.write_file(config_path, content)?;
                Ok(())
            },
            SelectedFile::Sql { collection, filename, scope } => {
                let relative_path = PathBuf::from(collection).join(filename);
                let full_path = self.get_scoped_path(*scope, relative_path)?;
                self.write_file(full_path, content)?;
                Ok(())
            },
            SelectedFile::Folder { .. } => {
                Err(anyhow::anyhow!("Cannot save content to a folder"))
            }
        }
    }

    fn copy_dir_recursive(&self, src: &PathBuf, dst: &PathBuf) -> Result<()> {
        fs::create_dir_all(dst)?;
        
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if ty.is_dir() {
                self.copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
                
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&dst_path, fs::Permissions::from_mode(0o600))?;
                }
            }
        }
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(dst, fs::Permissions::from_mode(0o700))?;
        }
        
        Ok(())
    }

    pub fn delete_file_or_folder(&self, name: &str, is_folder: bool, scope: CollectionScope) -> Result<()> {
        let base_path = self.get_base_path(scope);
        let target_path = base_path.join(name);
    
        if !target_path.exists() {
            return Err(anyhow::anyhow!("File or folder does not exist"));
        }
    
        if is_folder {
            fs::remove_dir_all(target_path)?;
        } else {
            fs::remove_file(target_path)?;
        }
    
        Ok(())
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

    if file_item.contains(CONFIG_FILE_NAME) {
        Some(SelectedFile::Config(scope))
    } else if selected.len() == 1 {
        let folder_name = file_item.split(" (").next()?;
        Some(SelectedFile::Folder {
            name: folder_name.to_string(),
            scope
        })
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

pub fn get_selected_folder_context(selected: &[String]) -> Option<(String, CollectionScope)> {
    if selected.is_empty() {
        return None;
    }
    
    let scope = if selected[0].contains("(user)") {
        CollectionScope::User
    } else {
        CollectionScope::Cwd
    };
    
    if selected.len() == 1 {
        let folder_name = selected[0].split(" (").next()?;
        return Some((folder_name.to_string(), scope));
    }
    
    if selected.len() == 2 && selected[1].ends_with(".sql") {
        let folder_name = selected[0].split(" (").next()?;
        return Some((folder_name.to_string(), scope));
    }
    
    None
}