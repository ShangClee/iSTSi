use loco_rs::prelude::*;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("tokens")
        .add("/balances", get(balances))
        .add("/transfer", post(transfer))
        .add("/mint", post(mint))
        .add("/burn", post(burn))
}

async fn balances() -> Result<Json<&'static str>> {
    // TODO: Implement token balances
    format::json("Token balances endpoint - to be implemented")
}

async fn transfer() -> Result<Json<&'static str>> {
    // TODO: Implement token transfer
    format::json("Token transfer endpoint - to be implemented")
}

async fn mint() -> Result<Json<&'static str>> {
    // TODO: Implement token mint
    format::json("Token mint endpoint - to be implemented")
}

async fn burn() -> Result<Json<&'static str>> {
    // TODO: Implement token burn
    format::json("Token burn endpoint - to be implemented")
}