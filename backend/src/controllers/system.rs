use loco_rs::prelude::*;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("system")
        .add("/overview", get(overview))
        .add("/health", get(health))
        .add("/version", get(version))
}

async fn overview() -> Result<Json<&'static str>> {
    // TODO: Implement system overview
    format::json("System overview endpoint - to be implemented")
}

async fn health() -> Result<Json<&'static str>> {
    // TODO: Implement health check
    format::json("Health check endpoint - to be implemented")
}

async fn version() -> Result<Json<&'static str>> {
    // TODO: Implement version info
    format::json("Version endpoint - to be implemented")
}