use std::path::PathBuf;

const USER_CONFIG_ENV_VAR: &str = "SQLI_CONFIG_DIR";
const WORKSPACE_ENV_VAR: &str = "SQLI_WORKSPACE_DIR";

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

    pub fn from_env() -> Self {
        let user_dir = std::env::var(USER_CONFIG_ENV_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("~/.config"))
                    .join("sqli")
            });

        let workspace_dir = std::env::var(WORKSPACE_ENV_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./sqli"));

        Self {
            user_dir,
            workspace_dir,
        }
    }
}