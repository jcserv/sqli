use std::fs;
use anyhow::Result;
use std::path::PathBuf;

use crate::collection::CollectionScope;

pub fn create_file_or_folder(name: &str, is_folder: bool, scope: CollectionScope) -> Result<()> {
    let base_path = match scope {
        CollectionScope::User => dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("sqli"),
        CollectionScope::Cwd => PathBuf::from("./sqli"),
    };

    // Create base directory if it doesn't exist
    if !base_path.exists() {
        fs::create_dir_all(&base_path)?;
    }

    let target_path = base_path.join(name);

    if target_path.exists() {
        return Err(anyhow::anyhow!("File or folder already exists"));
    }

    if is_folder {
        fs::create_dir_all(&target_path)?;
    } else {
        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        // Create empty file
        fs::write(&target_path, "")?;
    }

    // Set proper permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if is_folder { 0o700 } else { 0o600 };
        fs::set_permissions(&target_path, fs::Permissions::from_mode(mode))?;
    }

    Ok(())
}

pub fn rename_file_or_folder(old_name: &str, new_name: &str, current_scope: CollectionScope, new_scope: CollectionScope) -> Result<()> {
    let old_base_path = match current_scope {
        CollectionScope::User => dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("sqli"),
        CollectionScope::Cwd => PathBuf::from("./sqli"),
    };

    let new_base_path = match new_scope {
        CollectionScope::User => dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("sqli"),
        CollectionScope::Cwd => PathBuf::from("./sqli"),
    };

    let old_path = old_base_path.join(old_name);
    let new_path = new_base_path.join(new_name);

    if !old_path.exists() {
        return Err(anyhow::anyhow!("Source file or folder does not exist"));
    }

    if new_path.exists() {
        return Err(anyhow::anyhow!("Target file or folder already exists"));
    }

    // Create parent directory if needed
    if let Some(parent) = new_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // If moving between different scopes, we need to copy+delete
    if current_scope != new_scope {
        if old_path.is_dir() {
            copy_dir_recursive(&old_path, &new_path)?;
            fs::remove_dir_all(&old_path)?;
        } else {
            fs::copy(&old_path, &new_path)?;
            fs::remove_file(&old_path)?;
        }
    } else {
        // Same scope, can just rename
        fs::rename(&old_path, &new_path)?;
    }

    // Set proper permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if new_path.is_dir() { 0o700 } else { 0o600 };
        fs::set_permissions(&new_path, fs::Permissions::from_mode(mode))?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
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