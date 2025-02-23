use anyhow::Result;
use std::collections::HashMap;
use tui_tree_widget::TreeItem;

use crate::file::FileSystem;

#[derive(Debug)]
pub enum SelectedFile {
    Config(CollectionScope),
    Sql {
        collection: String,
        filename: String,
        scope: CollectionScope
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectionScope {
    User,
    Cwd,
}

impl CollectionScope {
    fn as_str(&self) -> &str {
        match self {
            CollectionScope::User => "(user)",
            CollectionScope::Cwd => "(cwd)",
        }
    }
}

pub struct Collection {
    pub name: String,
    pub files: Vec<String>,
    pub scope: CollectionScope,
}

pub fn load_collections() -> Result<Vec<Collection>> {
    let fs = FileSystem::new()?;
    let mut collections = Vec::new();
    
    let user_dir = fs.get_scoped_path(CollectionScope::User, "")?;
    if user_dir.exists() {
        fs.load_collections_from_dir(&user_dir, &mut collections, CollectionScope::User)?;
    }
    
    let local_dir = fs.get_scoped_path(CollectionScope::Cwd, "")?;
    if local_dir.exists() {
        fs.load_collections_from_dir(&local_dir, &mut collections, CollectionScope::Cwd)?;
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
    let mut tree_items = Vec::new();
    let fs = match FileSystem::new() {
        Ok(fs) => fs,
        Err(_) => return Vec::new(),
    };

    for scope in [CollectionScope::User, CollectionScope::Cwd] {
        if let Ok(config_path) = fs.get_scoped_path(scope, "config.yaml") {
            if config_path.exists() {
                let config_name = format!("{} {}", "config.yaml", scope.as_str());
                tree_items.push(TreeItem::new_leaf(config_name.clone(), config_name));
            }
        }
    }

    let mut user_collections = Vec::new();
    let mut cwd_collections = Vec::new();

    for collection in collections {
        match collection.scope {
            CollectionScope::User => user_collections.push(collection),
            CollectionScope::Cwd => cwd_collections.push(collection),
        }
    }

    for collections in [user_collections, cwd_collections] {
        for collection in collections {
            let collection_name = format!("{} {}", collection.name, collection.scope.as_str());
            let children: Vec<TreeItem<String>> = collection.files.iter()
                .map(|file| TreeItem::new_leaf(file.clone(), file.clone()))
                .collect();
            
            if let Ok(item) = TreeItem::new(collection_name.clone(), collection_name, children) {
                tree_items.push(item);
            }
        }
    }

    tree_items
}