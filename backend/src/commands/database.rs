use loco_rs::prelude::*;
use clap::{Parser, Subcommand};

use crate::seeders::DatabaseSeeder;
use crate::utils::database::DatabaseBackup;

#[derive(Parser)]
pub struct DatabaseCommand {
    #[command(subcommand)]
    pub command: DatabaseSubcommand,
}

#[derive(Subcommand)]
pub enum DatabaseSubcommand {
    /// Seed the database with development data
    Seed {
        /// Force seeding even if data already exists
        #[arg(long)]
        force: bool,
    },
    /// Clear all seeded data
    Clear {
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
    /// Create a database backup
    Backup {
        /// Backup directory path
        #[arg(long, default_value = "./backups")]
        path: String,
        /// Retention period in days
        #[arg(long, default_value = "30")]
        retention_days: u32,
    },
    /// Restore database from backup
    Restore {
        /// Backup file path
        #[arg(long)]
        file: String,
    },
    /// List available backups
    ListBackups {
        /// Backup directory path
        #[arg(long, default_value = "./backups")]
        path: String,
    },
    /// Show database status and statistics
    Status,
}

impl DatabaseCommand {
    pub async fn run(&self, ctx: &AppContext) -> Result<()> {
        match &self.command {
            DatabaseSubcommand::Seed { force } => {
                self.seed_database(ctx, *force).await
            }
            DatabaseSubcommand::Clear { yes } => {
                self.clear_database(ctx, *yes).await
            }
            DatabaseSubcommand::Backup { path, retention_days } => {
                self.backup_database(ctx, path, *retention_days).await
            }
            DatabaseSubcommand::Restore { file } => {
                self.restore_database(ctx, file).await
            }
            DatabaseSubcommand::ListBackups { path } => {
                self.list_backups(path).await
            }
            DatabaseSubcommand::Status => {
                self.show_status(ctx).await
            }
        }
    }

    async fn seed_database(&self, ctx: &AppContext, force: bool) -> Result<()> {
        let db = &ctx.db;

        // Check if already seeded
        if !force && DatabaseSeeder::is_seeded(db).await? {
            println!("Database is already seeded. Use --force to reseed.");
            return Ok(());
        }

        if force && DatabaseSeeder::is_seeded(db).await? {
            println!("Clearing existing data before reseeding...");
            DatabaseSeeder::clear_all(db).await?;
        }

        println!("Seeding database with development data...");
        DatabaseSeeder::seed_development(db).await?;
        println!("Database seeding completed successfully!");

        Ok(())
    }

    async fn clear_database(&self, ctx: &AppContext, skip_confirmation: bool) -> Result<()> {
        if !skip_confirmation {
            print!("Are you sure you want to clear all database data? This cannot be undone. (y/N): ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            
            if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
                println!("Operation cancelled.");
                return Ok(());
            }
        }

        let db = &ctx.db;
        println!("Clearing all database data...");
        DatabaseSeeder::clear_all(db).await?;
        println!("Database cleared successfully!");

        Ok(())
    }

    async fn backup_database(&self, ctx: &AppContext, backup_path: &str, retention_days: u32) -> Result<()> {
        let database_url = &ctx.config.database.uri;
        
        println!("Creating database backup...");
        let backup_file = DatabaseBackup::create_automated_backup(
            database_url,
            Some(backup_path),
            retention_days,
        ).await?;
        
        println!("Backup created successfully: {}", backup_file);
        Ok(())
    }

    async fn restore_database(&self, ctx: &AppContext, backup_file: &str) -> Result<()> {
        let database_url = &ctx.config.database.uri;
        
        println!("Restoring database from backup: {}", backup_file);
        DatabaseBackup::restore_backup(database_url, backup_file).await?;
        println!("Database restored successfully!");
        
        Ok(())
    }

    async fn list_backups(&self, backup_path: &str) -> Result<()> {
        let backups = DatabaseBackup::list_backups(Some(backup_path)).await?;
        
        if backups.is_empty() {
            println!("No backups found in {}", backup_path);
            return Ok(());
        }

        println!("Available backups in {}:", backup_path);
        println!("{:<30} {:<20} {:<15}", "Filename", "Created At", "Size");
        println!("{}", "-".repeat(65));
        
        for backup in backups {
            let size_mb = backup.size_bytes as f64 / 1024.0 / 1024.0;
            println!(
                "{:<30} {:<20} {:<15.2} MB",
                backup.filename,
                backup.created_at.format("%Y-%m-%d %H:%M:%S"),
                size_mb
            );
        }
        
        Ok(())
    }

    async fn show_status(&self, ctx: &AppContext) -> Result<()> {
        use crate::models::{user::User, kyc_record::KycRecord, token_balance::TokenBalance, operation::Operation};
        
        let db = &ctx.db;
        
        println!("Database Status");
        println!("===============");
        
        // Check if seeded
        let is_seeded = DatabaseSeeder::is_seeded(db).await?;
        println!("Seeded: {}", if is_seeded { "Yes" } else { "No" });
        
        // User statistics
        let (_, total_users) = User::list(db, 1, 1).await?;
        println!("Total Users: {}", total_users);
        
        // KYC statistics
        let (_, total_kyc) = KycRecord::list(db, 1, 1).await?;
        println!("Total KYC Records: {}", total_kyc);
        
        // Token balance statistics
        let balance_summary = TokenBalance::get_balances_summary(db).await?;
        println!("Token Balances:");
        for (token_type, total_balance, user_count) in balance_summary {
            println!("  {}: {} (across {} users)", token_type, total_balance, user_count);
        }
        
        // Operation statistics
        let op_stats = Operation::get_statistics(db).await?;
        println!("Operations:");
        println!("  Total: {}", op_stats.total_operations);
        println!("  Pending: {}", op_stats.pending_operations);
        println!("  Completed: {}", op_stats.completed_operations);
        println!("  Failed: {}", op_stats.failed_operations);
        
        Ok(())
    }
}