use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tui_tree_widget::TreeItem;

use crate::file::load_collections_from_dir;

pub struct Collection {
    pub name: String,
    pub files: Vec<String>,
}

pub fn load_collections() -> Result<Vec<Collection>> {
    let mut collections = Vec::new();
    
    if let Some(config_dir) = dirs::config_dir() {
        let user_collections_dir = config_dir.join("sqli").join("collections");
        if user_collections_dir.exists() {
            load_collections_from_dir(&user_collections_dir, &mut collections)?;
        }
    }
    
    let local_collections_dir = PathBuf::from("./sqli");
    if local_collections_dir.exists() {
        load_collections_from_dir(&local_collections_dir, &mut collections)?;
    }
    
    Ok(collections)
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