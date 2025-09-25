pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_users;
mod m20240101_000002_create_kyc_records;
mod m20240101_000003_create_token_balances;
mod m20240101_000004_create_operations;
mod m20240101_000005_create_audit_logs;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_users::Migration),
            Box::new(m20240101_000002_create_kyc_records::Migration),
            Box::new(m20240101_000003_create_token_balances::Migration),
            Box::new(m20240101_000004_create_operations::Migration),
            Box::new(m20240101_000005_create_audit_logs::Migration),
        ]
    }
}