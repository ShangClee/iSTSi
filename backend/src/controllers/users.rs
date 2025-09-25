use loco_rs::prelude::*;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("users")
        .add("/", get(list))
        .add("/:id", get(get_one))
        .add("/", post(add))
        .add("/:id", put(update))
        .add("/:id", delete(remove))
}

async fn list() -> Result<Json<&'static str>> {
    // TODO: Implement user listing
    format::json("User list endpoint - to be implemented")
}

async fn get_one() -> Result<Json<&'static str>> {
    // TODO: Implement get user by id
    format::json("Get user endpoint - to be implemented")
}

async fn add() -> Result<Json<&'static str>> {
    // TODO: Implement add user
    format::json("Add user endpoint - to be implemented")
}

async fn update() -> Result<Json<&'static str>> {
    // TODO: Implement update user
    format::json("Update user endpoint - to be implemented")
}

async fn remove() -> Result<Json<&'static str>> {
    // TODO: Implement remove user
    format::json("Remove user endpoint - to be implemented")
}