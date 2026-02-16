use std::fs;
use std::path::Path;

/// Create a .bak backup of the file before overwriting.
pub fn create_backup(path: &Path) -> Result<(), String> {
    if path.exists() {
        let backup_path = path.with_extension(
            format!(
                "{}.bak",
                path.extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default()
            ),
        );
        fs::copy(path, &backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
    }
    Ok(())
}

/// Write content to a file, creating a backup first.
pub fn safe_write(path: &Path, content: &str) -> Result<(), String> {
    create_backup(path)?;
    fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}
