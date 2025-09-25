use loco_rs::prelude::*;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("reserves")
        .add("/status", get(status))
        .add("/proof", get(proof))
        .add("/ratio", get(ratio))
        .add("/audit", get(audit))
}

async fn status() -> Result<Json<&'static str>> {
    // TODO: Implement reserve status
    format::json("Reserve status endpoint - to be implemented")
}

async fn proof() -> Result<Json<&'static str>> {
    // TODO: Implement proof of reserves
    format::json("Proof of reserves endpoint - to be implemented")
}

async fn ratio() -> Result<Json<&'static str>> {
    // TODO: Implement reserve ratio
    format::json("Reserve ratio endpoint - to be implemented")
}

async fn audit() -> Result<Json<&'static str>> {
    // TODO: Implement reserve audit
    format::json("Reserve audit endpoint - to be implemented")
}