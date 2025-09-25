use loco_rs::prelude::*;
use std::process::Command;
use std::path::Path;
use chrono::{DateTime, Utc};

pub struct DatabaseBackup;

impl DatabaseBackup {
    /// Create a database backup
    pub async fn create_backup(
        database_url: &str,
        backup_path: Option<&str>,
    ) -> Result<String> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("bitcoin_custody_backup_{}.sql", timestamp);
        
        let backup_dir = backup_path.unwrap_or("./backups");
        std::fs::create_dir_all(backup_dir)
            .map_err(|e| Error::string(&format!("Failed to create backup directory: {}", e)))?;
        
        let backup_file_path = Path::new(backup_dir).join(&backup_filename);
        
        tracing::info!("Creating database backup: {}", backup_file_path.display());
        
        // Parse database URL to extract connection parameters
        let db_params = Self::parse_database_url(database_url)?;
        
        // Execute pg_dump command
        let output = Command::new("pg_dump")
            .arg("--host").arg(&db_params.host)
            .arg("--port").arg(&db_params.port.to_string())
            .arg("--username").arg(&db_params.username)
            .arg("--dbname").arg(&db_params.database)
            .arg("--no-password")
            .arg("--verbose")
            .arg("--clean")
            .arg("--if-exists")
            .arg("--create")
            .arg("--file").arg(&backup_file_path)
            .env("PGPASSWORD", &db_params.password)
            .output()
            .map_err(|e| Error::string(&format!("Failed to execute pg_dump: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(Error::string(&format!("pg_dump failed: {}", error_msg)));
        }
        
        tracing::info!("Database backup created successfully: {}", backup_file_path.display());
        Ok(backup_file_path.to_string_lossy().to_string())
    }
    
    /// Restore database from backup
    pub async fn restore_backup(
        database_url: &str,
        backup_file_path: &str,
    ) -> Result<()> {
        if !Path::new(backup_file_path).exists() {
            return Err(Error::string(&format!("Backup file not found: {}", backup_file_path)));
        }
        
        tracing::info!("Restoring database from backup: {}", backup_file_path);
        
        // Parse database URL to extract connection parameters
        let db_params = Self::parse_database_url(database_url)?;
        
        // Execute psql command to restore
        let output = Command::new("psql")
            .arg("--host").arg(&db_params.host)
            .arg("--port").arg(&db_params.port.to_string())
            .arg("--username").arg(&db_params.username)
            .arg("--dbname").arg(&db_params.database)
            .arg("--no-password")
            .arg("--file").arg(backup_file_path)
            .env("PGPASSWORD", &db_params.password)
            .output()
            .map_err(|e| Error::string(&format!("Failed to execute psql: {}", e)))?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(Error::string(&format!("Database restore failed: {}", error_msg)));
        }
        
        tracing::info!("Database restored successfully from: {}", backup_file_path);
        Ok(())
    }
    
    /// Create automated backup with retention policy
    pub async fn create_automated_backup(
        database_url: &str,
        backup_path: Option<&str>,
        retention_days: u32,
    ) -> Result<String> {
        // Create new backup
        let backup_file = Self::create_backup(database_url, backup_path).await?;
        
        // Clean up old backups based on retention policy
        Self::cleanup_old_backups(backup_path.unwrap_or("./backups"), retention_days).await?;
        
        Ok(backup_file)
    }
    
    /// Clean up old backup files based on retention policy
    async fn cleanup_old_backups(backup_dir: &str, retention_days: u32) -> Result<()> {
        let backup_path = Path::new(backup_dir);
        if !backup_path.exists() {
            return Ok(());
        }
        
        let cutoff_time = Utc::now() - chrono::Duration::days(retention_days as i64);
        
        let entries = std::fs::read_dir(backup_path)
            .map_err(|e| Error::string(&format!("Failed to read backup directory: {}", e)))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| Error::string(&format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "sql") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(created) = metadata.created() {
                        let created_time: DateTime<Utc> = created.into();
                        if created_time < cutoff_time {
                            if let Err(e) = std::fs::remove_file(&path) {
                                tracing::warn!("Failed to remove old backup file {}: {}", path.display(), e);
                            } else {
                                tracing::info!("Removed old backup file: {}", path.display());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse PostgreSQL database URL
    fn parse_database_url(database_url: &str) -> Result<DatabaseParams> {
        // Parse URL like: postgres://username:password@host:port/database
        let url = database_url.strip_prefix("postgres://")
            .or_else(|| database_url.strip_prefix("postgresql://"))
            .ok_or_else(|| Error::string("Invalid database URL format"))?;
        
        let (credentials, host_db) = url.split_once('@')
            .ok_or_else(|| Error::string("Invalid database URL format: missing @"))?;
        
        let (username, password) = credentials.split_once(':')
            .ok_or_else(|| Error::string("Invalid database URL format: missing password"))?;
        
        let (host_port, database) = host_db.split_once('/')
            .ok_or_else(|| Error::string("Invalid database URL format: missing database name"))?;
        
        let (host, port_str) = if host_port.contains(':') {
            host_port.split_once(':')
                .ok_or_else(|| Error::string("Invalid database URL format: invalid port"))?
        } else {
            (host_port, "5432")
        };
        
        let port = port_str.parse::<u16>()
            .map_err(|_| Error::string("Invalid port number"))?;
        
        Ok(DatabaseParams {
            host: host.to_string(),
            port,
            username: username.to_string(),
            password: password.to_string(),
            database: database.to_string(),
        })
    }
    
    /// List available backup files
    pub async fn list_backups(backup_path: Option<&str>) -> Result<Vec<BackupInfo>> {
        let backup_dir = backup_path.unwrap_or("./backups");
        let backup_path = Path::new(backup_dir);
        
        if !backup_path.exists() {
            return Ok(Vec::new());
        }
        
        let entries = std::fs::read_dir(backup_path)
            .map_err(|e| Error::string(&format!("Failed to read backup directory: {}", e)))?;
        
        let mut backups = Vec::new();
        
        for entry in entries {
            let entry = entry.map_err(|e| Error::string(&format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "sql") {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(created) = metadata.created() {
                            let created_time: DateTime<Utc> = created.into();
                            let size = metadata.len();
                            
                            backups.push(BackupInfo {
                                filename: filename.to_string(),
                                path: path.to_string_lossy().to_string(),
                                created_at: created_time,
                                size_bytes: size,
                            });
                        }
                    }
                }
            }
        }
        
        // Sort by creation time, newest first
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(backups)
    }
}

#[derive(Debug)]
struct DatabaseParams {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}

#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub filename: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_database_url() {
        let url = "postgres://user:pass@localhost:5432/testdb";
        let params = DatabaseBackup::parse_database_url(url).unwrap();
        
        assert_eq!(params.host, "localhost");
        assert_eq!(params.port, 5432);
        assert_eq!(params.username, "user");
        assert_eq!(params.password, "pass");
        assert_eq!(params.database, "testdb");
    }
    
    #[test]
    fn test_parse_database_url_default_port() {
        let url = "postgres://user:pass@localhost/testdb";
        let params = DatabaseBackup::parse_database_url(url).unwrap();
        
        assert_eq!(params.host, "localhost");
        assert_eq!(params.port, 5432);
        assert_eq!(params.database, "testdb");
    }
}