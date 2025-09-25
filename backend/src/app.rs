use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Hooks},
    boot::{create_app, BootResult, StartMode},
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
    worker::Processor,
    Result,
};
use migration::Migrator;
use sea_orm::DatabaseConnection;

use crate::{
    controllers,
    // workers,
};

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment).await
    }

    async fn truncate(db: &DatabaseConnection) -> Result<()> {
        use crate::seeders::DatabaseSeeder;
        
        // Clear all data for testing
        DatabaseSeeder::clear_all(db).await?;
        Ok(())
    }

    async fn seed(db: &DatabaseConnection, _base: &std::path::Path) -> Result<()> {
        use crate::seeders::DatabaseSeeder;
        
        // Only seed in development environment
        if std::env::var("LOCO_ENV").unwrap_or_else(|_| "development".to_string()) == "development" {
            if !DatabaseSeeder::is_seeded(db).await? {
                tracing::info!("Seeding development database...");
                DatabaseSeeder::seed_development(db).await?;
                tracing::info!("Database seeding completed");
            } else {
                tracing::info!("Database already seeded, skipping");
            }
        }
        
        Ok(())
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .prefix("/api")
            .add_route(controllers::auth::routes())
            .add_route(controllers::users::routes())
            .add_route(controllers::integration::routes())
            .add_route(controllers::kyc::routes())
            .add_route(controllers::tokens::routes())
            .add_route(controllers::reserves::routes())
            .add_route(controllers::system::routes())
    }

    fn connect_workers<'a>(_p: &'a mut Processor, _ctx: &'a AppContext) {
        // TODO: Register workers when implemented
        // use crate::workers;
        // p.register(workers::event_monitor::EventMonitorWorker::build(ctx));
        // p.register(workers::reconciliation::ReconciliationWorker::build(ctx));
        // p.register(workers::proof_generation::ProofGenerationWorker::build(ctx));
    }

    fn register_tasks(_tasks: &mut Tasks) {
        // Database management commands are handled through CLI
        // Use: cargo loco task database --help
    }
}