use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct UserSettings {
    pub user_dir: PathBuf,
    pub workspace_dir: PathBuf,
}

impl Default for UserSettings {
    fn default() -> Self {
        let user_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("sqli");
            
        let workspace_dir = PathBuf::from("./sqli");
        
        Self {
            user_dir,
            workspace_dir,
        }
    }
}

impl UserSettings {
    pub fn new(user_dir: PathBuf, workspace_dir: PathBuf) -> Self {
        Self {
            user_dir,
            workspace_dir,
        }
    }
}