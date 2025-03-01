use std::{fs::{self, File}, io::Write};

use tempfile::TempDir;

pub struct TestEnv {
    pub temp_dir: TempDir,
    original_config_dir: Option<String>,
    original_workspace_dir: Option<String>,
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl TestEnv {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join(".sqli")).unwrap();        
        std::env::set_current_dir(&temp_dir).unwrap();

        let original_config_dir = std::env::var("SQLI_CONFIG_DIR").ok();
        let original_workspace_dir = std::env::var("SQLI_WORKSPACE_DIR").ok();

        Self { 
            temp_dir, 
            original_config_dir,
            original_workspace_dir 
        }
    }

    #[allow(dead_code)]
    pub fn setup_test_env(&self) {
        std::env::set_var("SQLI_CONFIG_DIR", self.temp_dir.path().join("sqli").display().to_string());
        std::env::set_var("SQLI_WORKSPACE_DIR", self.temp_dir.path().join(".sqli").display().to_string());
    }

    #[allow(dead_code)]
    pub fn create_sql_file(&self, name: &str, contents: &str) -> std::io::Result<()> {
        let mut file = File::create(self.temp_dir.path().join(name))?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()
    }

    pub fn create_config(&self, contents: &str) -> std::io::Result<()> {
        let config_dir = self.temp_dir.path().join("sqli");
        fs::create_dir_all(&config_dir)?;
        
        let mut file = File::create(config_dir.join("config.yaml"))?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()
    }

    pub fn create_collection(&self, collection_name: &str, files: &[(&str, &str)]) -> std::io::Result<()> {
        let collection_dir = self.temp_dir.path().join("sqli").join(collection_name);
        fs::create_dir_all(&collection_dir)?;
        
        for (file_name, content) in files {
            let mut file = File::create(collection_dir.join(file_name))?;
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
        }
        
        Ok(())
    }

    #[allow(dead_code)]
    pub fn create_nested_collection(&self, path: &str, files: &[(&str, &str)]) -> std::io::Result<()> {
        let full_path = self.temp_dir.path().join("sqli").join(path);
        fs::create_dir_all(&full_path)?;
        
        for (file_name, content) in files {
            let mut file = File::create(full_path.join(file_name))?;
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
        }
        
        Ok(())
    }

    #[allow(dead_code)]
    pub fn path(&self, rel_path: &str) -> std::path::PathBuf {
        self.temp_dir.path().join(rel_path)
    }

    #[allow(dead_code)]
    pub fn ensure_dir(&self, rel_path: &str) -> std::io::Result<()> {
        let path = self.temp_dir.path().join(rel_path);
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn file_exists(&self, rel_path: &str) -> bool {
        self.temp_dir.path().join(rel_path).exists()
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        match &self.original_config_dir {
            Some(val) => std::env::set_var("SQLI_CONFIG_DIR", val),
            None => std::env::remove_var("SQLI_CONFIG_DIR"),
        }
        
        match &self.original_workspace_dir {
            Some(val) => std::env::set_var("SQLI_WORKSPACE_DIR", val),
            None => std::env::remove_var("SQLI_WORKSPACE_DIR"),
        }
    }
}

#[allow(dead_code)]
pub fn contains_any_of(text: &str, options: &[&str]) -> bool {
    options.iter().any(|&option| text.contains(option))
}

#[allow(dead_code)]
pub fn create_standard_test_env() -> TestEnv {
    let env = TestEnv::new();
    
    env.create_config(r#"
connections:
  - name: local
    conn: postgresql
    host: localhost
    port: 5432
    database: testdb
    user: postgres
  - name: staging
    conn: postgresql
    host: staging-db.example.com
    port: 5432
    database: staging_db
    user: app_user
"#).unwrap();

    env.create_collection("users", &[
        ("list.sql", "SELECT * FROM users;"),
        ("get.sql", "SELECT * FROM users WHERE id = $1;"),
        ("create.sql", "INSERT INTO users (name, email) VALUES ($1, $2);")
    ]).unwrap();
    
    env.create_collection("products", &[
        ("list.sql", "SELECT * FROM products;"),
        ("get.sql", "SELECT * FROM products WHERE id = $1;"),
        ("search.sql", "SELECT * FROM products WHERE name LIKE $1;")
    ]).unwrap();
    
    env.create_nested_collection("database/schemas/public", &[
        ("tables.sql", "SELECT * FROM information_schema.tables;"),
        ("columns.sql", "SELECT * FROM information_schema.columns WHERE table_name = $1;")
    ]).unwrap();
    
    env
}