use loco_rs::prelude::*;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("kyc")
        .add("/", get(list))
        .add("/:id", get(get_one))
        .add("/", post(add))
        .add("/:id", put(update))
        .add("/:id/approve", post(approve))
}

async fn list() -> Result<Json<&'static str>> {
    // TODO: Implement KYC record listing
    format::json("KYC list endpoint - to be implemented")
}

async fn get_one() -> Result<Json<&'static str>> {
    // TODO: Implement get KYC record by id
    format::json("Get KYC record endpoint - to be implemented")
}

async fn add() -> Result<Json<&'static str>> {
    // TODO: Implement add KYC record
    format::json("Add KYC record endpoint - to be implemented")
}

async fn update() -> Result<Json<&'static str>> {
    // TODO: Implement update KYC record
    format::json("Update KYC record endpoint - to be implemented")
}

async fn approve() -> Result<Json<&'static str>> {
    // TODO: Implement approve KYC record
    format::json("Approve KYC record endpoint - to be implemented")
}