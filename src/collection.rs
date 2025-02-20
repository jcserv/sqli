use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use tui_tree_widget::TreeItem;

pub struct Collection {
    pub name: String,
    pub files: Vec<String>,
}

pub fn load_collections() -> Result<Vec<Collection>> {
    let mut collections = Vec::new();
    
    // Check user-level collections first (~/.config/sqli/collections)
    if let Some(config_dir) = dirs::config_dir() {
        let user_collections_dir = config_dir.join("sqli").join("collections");
        if user_collections_dir.exists() {
            load_collections_from_dir(&user_collections_dir, &mut collections)?;
        }
    }
    
    // Then check local collections (./sqli)
    let local_collections_dir = PathBuf::from("./sqli");
    if local_collections_dir.exists() {
        load_collections_from_dir(&local_collections_dir, &mut collections)?;
    }
    
    Ok(collections)
}

fn load_collections_from_dir(dir: &Path, collections: &mut Vec<Collection>) -> Result<()> {
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

pub fn collections_to_hashmap(collections: &[Collection]) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    for collection in collections {
        map.insert(collection.name.clone(), collection.files.clone());
    }
    map
}

pub fn build_collection_tree<'a>(collections: &[Collection]) -> Vec<TreeItem<'a, String>> {
    collections.iter()
        .map(|collection| {
            let children: Vec<TreeItem<String>> = collection.files.iter()
                .map(|file| {
                    TreeItem::new_leaf(file.clone(), file.clone())
                })
                .collect();
            
            TreeItem::new(
                collection.name.clone(),
                collection.name.clone(),
                children
            ).expect("all item identifiers are unique")
        })
        .collect()
}

// Function to load SQL content from a file
pub fn load_sql_content(collection_name: &str, file_name: &str) -> Result<String> {
    let local_path = PathBuf::from("./sqli")
        .join(collection_name)
        .join(file_name);
    
    if local_path.exists() {
        return Ok(fs::read_to_string(local_path)?);
    }
    
    if let Some(config_dir) = dirs::config_dir() {
        let user_path = config_dir
            .join("sqli")
            .join("collections")
            .join(collection_name)
            .join(file_name);
        
        if user_path.exists() {
            return Ok(fs::read_to_string(user_path)?);
        }
    }
    
    anyhow::bail!("SQL file not found: {}/{}", collection_name, file_name)
}