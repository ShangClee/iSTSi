use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBackup {
    pub id: String,
    pub timestamp: u64,
    pub environment: String,
    pub config_files: HashMap<String, String>, // file_path -> content_hash
    pub metadata: BackupMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub created_by: String,
    pub backup_type: BackupType,
    pub description: String,
    pub size_bytes: u64,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    Scheduled,
    Manual,
    PreDeployment,
    Emergency,
}

#[derive(Debug, Clone)]
pub struct ConfigBackupService {
    backup_directory: PathBuf,
    retention_days: u32,
    encryption_enabled: bool,
}

impl ConfigBackupService {
    pub fn new(backup_directory: &str, retention_days: u32, encryption_enabled: bool) -> Result<Self> {
        let backup_dir = PathBuf::from(backup_directory);
        
        // Create backup directory if it doesn't exist
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)
                .map_err(|e| Error::string(&format!("Failed to create backup directory: {}", e)))?;
        }

        Ok(Self {
            backup_directory: backup_dir,
            retention_days,
            encryption_enabled,
        })
    }

    /// Create a backup of all configuration files
    pub async fn create_backup(
        &self,
        environment: &str,
        backup_type: BackupType,
        description: String,
    ) -> Result<ConfigBackup> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let backup_id = format!("config_backup_{}_{}", environment, timestamp);
        
        // Define configuration files to backup
        let config_files = self.get_config_files_for_environment(environment)?;
        
        let mut backed_up_files = HashMap::new();
        let mut total_size = 0u64;
        
        // Create backup directory for this backup
        let backup_path = self.backup_directory.join(&backup_id);
        fs::create_dir_all(&backup_path)
            .map_err(|e| Error::string(&format!("Failed to create backup path: {}", e)))?;

        // Backup each configuration file
        for (file_path, source_path) in config_files {
            if let Ok(content) = fs::read_to_string(&source_path) {
                let content_hash = self.calculate_hash(&content);
                
                // Encrypt content if encryption is enabled
                let final_content = if self.encryption_enabled {
                    self.encrypt_content(&content)?
                } else {
                    content
                };
                
                // Write to backup directory
                let backup_file_path = backup_path.join(&file_path.replace('/', "_"));
                fs::write(&backup_file_path, &final_content)
                    .map_err(|e| Error::string(&format!("Failed to write backup file: {}", e)))?;
                
                backed_up_files.insert(file_path, content_hash);
                total_size += final_content.len() as u64;
            }
        }

        let metadata = BackupMetadata {
            created_by: whoami::username(),
            backup_type,
            description,
            size_bytes: total_size,
            file_count: backed_up_files.len(),
        };

        let backup = ConfigBackup {
            id: backup_id.clone(),
            timestamp,
            environment: environment.to_string(),
            config_files: backed_up_files,
            metadata,
        };

        // Save backup metadata
        let metadata_path = backup_path.join("backup_metadata.json");
        let metadata_json = serde_json::to_string_pretty(&backup)
            .map_err(|e| Error::string(&format!("Failed to serialize backup metadata: {}", e)))?;
        
        fs::write(metadata_path, metadata_json)
            .map_err(|e| Error::string(&format!("Failed to write backup metadata: {}", e)))?;

        tracing::info!("Configuration backup created: {} ({} files, {} bytes)", 
                      backup_id, backup.metadata.file_count, backup.metadata.size_bytes);

        Ok(backup)
    }

    /// Restore configuration from backup
    pub async fn restore_backup(&self, backup_id: &str, target_environment: &str) -> Result<()> {
        let backup_path = self.backup_directory.join(backup_id);
        
        if !backup_path.exists() {
            return Err(Error::string(&format!("Backup {} not found", backup_id)));
        }

        // Load backup metadata
        let metadata_path = backup_path.join("backup_metadata.json");
        let metadata_content = fs::read_to_string(metadata_path)
            .map_err(|e| Error::string(&format!("Failed to read backup metadata: {}", e)))?;
        
        let backup: ConfigBackup = serde_json::from_str(&metadata_content)
            .map_err(|e| Error::string(&format!("Failed to parse backup metadata: {}", e)))?;

        // Validate environment compatibility
        if backup.environment != target_environment {
            tracing::warn!("Restoring backup from {} to {} environment", 
                          backup.environment, target_environment);
        }

        // Create backup of current configuration before restore
        self.create_backup(target_environment, BackupType::Emergency, 
                          format!("Pre-restore backup before restoring {}", backup_id)).await?;

        // Restore each configuration file
        for (file_path, _hash) in &backup.config_files {
            let backup_file_path = backup_path.join(&file_path.replace('/', "_"));
            
            if backup_file_path.exists() {
                let content = fs::read_to_string(&backup_file_path)
                    .map_err(|e| Error::string(&format!("Failed to read backup file: {}", e)))?;
                
                // Decrypt content if encryption was enabled
                let final_content = if self.encryption_enabled {
                    self.decrypt_content(&content)?
                } else {
                    content
                };
                
                // Determine target path for restoration
                let target_path = self.get_target_path_for_file(file_path, target_environment)?;
                
                // Create parent directories if needed
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| Error::string(&format!("Failed to create parent directory: {}", e)))?;
                }
                
                // Write restored content
                fs::write(&target_path, final_content)
                    .map_err(|e| Error::string(&format!("Failed to restore file {}: {}", file_path, e)))?;
                
                tracing::info!("Restored configuration file: {}", file_path);
            }
        }

        tracing::info!("Configuration restore completed from backup: {}", backup_id);
        Ok(())
    }

    /// List available backups
    pub fn list_backups(&self, environment: Option<&str>) -> Result<Vec<ConfigBackup>> {
        let mut backups = Vec::new();
        
        if !self.backup_directory.exists() {
            return Ok(backups);
        }

        for entry in fs::read_dir(&self.backup_directory)
            .map_err(|e| Error::string(&format!("Failed to read backup directory: {}", e)))? {
            
            let entry = entry.map_err(|e| Error::string(&format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            
            if path.is_dir() {
                let metadata_path = path.join("backup_metadata.json");
                if metadata_path.exists() {
                    if let Ok(metadata_content) = fs::read_to_string(metadata_path) {
                        if let Ok(backup) = serde_json::from_str::<ConfigBackup>(&metadata_content) {
                            // Filter by environment if specified
                            if environment.is_none() || environment == Some(&backup.environment) {
                                backups.push(backup);
                            }
                        }
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(backups)
    }

    /// Clean up old backups based on retention policy
    pub async fn cleanup_old_backups(&self) -> Result<usize> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let retention_seconds = self.retention_days as u64 * 24 * 60 * 60;
        let cutoff_time = current_time - retention_seconds;
        
        let backups = self.list_backups(None)?;
        let mut cleaned_count = 0;
        
        for backup in backups {
            if backup.timestamp < cutoff_time {
                let backup_path = self.backup_directory.join(&backup.id);
                if backup_path.exists() {
                    fs::remove_dir_all(&backup_path)
                        .map_err(|e| Error::string(&format!("Failed to remove old backup: {}", e)))?;
                    
                    tracing::info!("Removed old backup: {} (age: {} days)", 
                                  backup.id, (current_time - backup.timestamp) / (24 * 60 * 60));
                    cleaned_count += 1;
                }
            }
        }
        
        Ok(cleaned_count)
    }

    /// Verify backup integrity
    pub fn verify_backup(&self, backup_id: &str) -> Result<bool> {
        let backup_path = self.backup_directory.join(backup_id);
        
        if !backup_path.exists() {
            return Err(Error::string(&format!("Backup {} not found", backup_id)));
        }

        // Load backup metadata
        let metadata_path = backup_path.join("backup_metadata.json");
        let metadata_content = fs::read_to_string(metadata_path)
            .map_err(|e| Error::string(&format!("Failed to read backup metadata: {}", e)))?;
        
        let backup: ConfigBackup = serde_json::from_str(&metadata_content)
            .map_err(|e| Error::string(&format!("Failed to parse backup metadata: {}", e)))?;

        // Verify each file exists and has correct hash
        for (file_path, expected_hash) in &backup.config_files {
            let backup_file_path = backup_path.join(&file_path.replace('/', "_"));
            
            if !backup_file_path.exists() {
                tracing::error!("Backup file missing: {}", file_path);
                return Ok(false);
            }
            
            let content = fs::read_to_string(&backup_file_path)
                .map_err(|e| Error::string(&format!("Failed to read backup file: {}", e)))?;
            
            // Decrypt if needed before hash verification
            let final_content = if self.encryption_enabled {
                self.decrypt_content(&content)?
            } else {
                content
            };
            
            let actual_hash = self.calculate_hash(&final_content);
            if &actual_hash != expected_hash {
                tracing::error!("Hash mismatch for file {}: expected {}, got {}", 
                               file_path, expected_hash, actual_hash);
                return Ok(false);
            }
        }
        
        tracing::info!("Backup verification successful: {}", backup_id);
        Ok(true)
    }

    /// Get configuration files for a specific environment
    fn get_config_files_for_environment(&self, environment: &str) -> Result<HashMap<String, PathBuf>> {
        let mut files = HashMap::new();
        
        // Backend configuration files
        let backend_config_path = PathBuf::from("backend/config");
        files.insert(
            format!("backend/config/{}.yaml", environment),
            backend_config_path.join(format!("{}.yaml", environment))
        );
        
        // Frontend environment files
        let frontend_env_path = PathBuf::from("frontend");
        files.insert(
            format!("frontend/.env.{}", environment),
            frontend_env_path.join(format!(".env.{}", environment))
        );
        
        // Soroban configuration
        let soroban_config_path = PathBuf::from("soroban/config");
        if soroban_config_path.exists() {
            files.insert(
                "soroban/config/network_config.toml".to_string(),
                soroban_config_path.join("network_config.toml")
            );
        }
        
        // Docker configuration
        files.insert(
            "docker-compose.yml".to_string(),
            PathBuf::from("docker-compose.yml")
        );
        
        if environment != "development" {
            files.insert(
                format!("docker-compose.{}.yml", environment),
                PathBuf::from(format!("docker-compose.{}.yml", environment))
            );
        }
        
        Ok(files)
    }

    /// Get target path for restoring a file
    fn get_target_path_for_file(&self, file_path: &str, _environment: &str) -> Result<PathBuf> {
        // For now, restore to the same path
        // In a more sophisticated system, you might want to adjust paths based on environment
        Ok(PathBuf::from(file_path))
    }

    /// Calculate hash of content
    fn calculate_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Encrypt content (placeholder implementation)
    fn encrypt_content(&self, content: &str) -> Result<String> {
        // In a real implementation, use proper encryption like AES-GCM
        // This is a simple XOR encryption for demonstration
        let key = "backup_encryption_key_change_in_production";
        let encrypted: Vec<u8> = content
            .bytes()
            .zip(key.bytes().cycle())
            .map(|(a, b)| a ^ b)
            .collect();
        
        use base64::{Engine as _, engine::general_purpose};
        Ok(general_purpose::STANDARD.encode(encrypted))
    }

    /// Decrypt content (placeholder implementation)
    fn decrypt_content(&self, encrypted_content: &str) -> Result<String> {
        let key = "backup_encryption_key_change_in_production";
        use base64::{Engine as _, engine::general_purpose};
        let encrypted_bytes = general_purpose::STANDARD.decode(encrypted_content)
            .map_err(|e| Error::string(&format!("Failed to decode encrypted content: {}", e)))?;
        
        let decrypted: Vec<u8> = encrypted_bytes
            .iter()
            .zip(key.bytes().cycle())
            .map(|(a, b)| a ^ b)
            .collect();
        
        String::from_utf8(decrypted)
            .map_err(|e| Error::string(&format!("Failed to decrypt content: {}", e)))
    }
}

/// Scheduled backup task
pub async fn run_scheduled_backup(
    service: &ConfigBackupService,
    environment: &str,
) -> Result<()> {
    tracing::info!("Running scheduled configuration backup for environment: {}", environment);
    
    let backup = service.create_backup(
        environment,
        BackupType::Scheduled,
        format!("Scheduled backup for {}", environment),
    ).await?;
    
    tracing::info!("Scheduled backup completed: {}", backup.id);
    
    // Clean up old backups
    let cleaned_count = service.cleanup_old_backups().await?;
    if cleaned_count > 0 {
        tracing::info!("Cleaned up {} old backups", cleaned_count);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let service = ConfigBackupService::new(
            temp_dir.path().to_str().unwrap(),
            30,
            false,
        ).unwrap();

        // This test would need actual config files to work properly
        // In a real test, you'd set up mock config files first
    }

    #[test]
    fn test_hash_calculation() {
        let service = ConfigBackupService::new("/tmp", 30, false).unwrap();
        let content = "test content";
        let hash1 = service.calculate_hash(content);
        let hash2 = service.calculate_hash(content);
        
        assert_eq!(hash1, hash2);
        
        let different_content = "different content";
        let hash3 = service.calculate_hash(different_content);
        assert_ne!(hash1, hash3);
    }
}