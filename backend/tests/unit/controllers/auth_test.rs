use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use bitcoin_custody_backend::{
    controllers::auth::{LoginRequest, RegisterRequest},
    models::user::{User, UserRole, KycStatus},
    app::App,
};
use loco_rs::testing;
use sea_orm::DatabaseConnection;
use serde_json::json;
use tower::ServiceExt;

async fn setup_test_app() -> (App, DatabaseConnection) {
    let config = loco_rs::config::Config::new("test").unwrap();
    let db = testing::db::setup(&config.database).await.unwrap();
    let app = App::new(config, db.clone()).await.unwrap();
    (app, db)
}

#[tokio::test]
async fn test_register_user_success() {
    let (app, _db) = setup_test_app().await;

    let register_request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        stellar_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
    };

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&register_request).unwrap()))
        .unwrap();

    let response = app.router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response_data["token"].is_string());
    assert_eq!(response_data["user"]["email"], "test@example.com");
    assert_eq!(response_data["user"]["kyc_status"], "pending");
}

#[tokio::test]
async fn test_register_user_duplicate_email() {
    let (app, db) = setup_test_app().await;

    // Create a user first
    let existing_user = User {
        id: uuid::Uuid::new_v4(),
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        stellar_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        role: UserRole::User,
        kyc_status: KycStatus::Pending,
        tier: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Insert user into database
    testing::db::seed_user(&db, &existing_user).await.unwrap();

    let register_request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        stellar_address: "GBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB".to_string(),
    };

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&register_request).unwrap()))
        .unwrap();

    let response = app.router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response_data["error"].as_str().unwrap().contains("already exists"));
}

#[tokio::test]
async fn test_login_success() {
    let (app, db) = setup_test_app().await;

    // Create and seed a user
    let password = "password123";
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
    
    let user = User {
        id: uuid::Uuid::new_v4(),
        email: "test@example.com".to_string(),
        password_hash,
        stellar_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        role: UserRole::User,
        kyc_status: KycStatus::Approved,
        tier: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    testing::db::seed_user(&db, &user).await.unwrap();

    let login_request = LoginRequest {
        email: "test@example.com".to_string(),
        password: password.to_string(),
    };

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&login_request).unwrap()))
        .unwrap();

    let response = app.router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response_data["token"].is_string());
    assert_eq!(response_data["user"]["email"], "test@example.com");
    assert_eq!(response_data["user"]["kyc_status"], "approved");
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let (app, _db) = setup_test_app().await;

    let login_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "wrongpassword".to_string(),
    };

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&login_request).unwrap()))
        .unwrap();

    let response = app.router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response_data["error"].as_str().unwrap().contains("Invalid credentials"));
}

#[tokio::test]
async fn test_logout_success() {
    let (app, db) = setup_test_app().await;

    // Create and seed a user
    let user = User {
        id: uuid::Uuid::new_v4(),
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        stellar_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        role: UserRole::User,
        kyc_status: KycStatus::Approved,
        tier: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    testing::db::seed_user(&db, &user).await.unwrap();

    // Generate a JWT token for the user
    let token = testing::auth::generate_test_token(&user).unwrap();

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/logout")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_current_user() {
    let (app, db) = setup_test_app().await;

    // Create and seed a user
    let user = User {
        id: uuid::Uuid::new_v4(),
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        stellar_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        role: UserRole::User,
        kyc_status: KycStatus::Approved,
        tier: 2,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    testing::db::seed_user(&db, &user).await.unwrap();

    // Generate a JWT token for the user
    let token = testing::auth::generate_test_token(&user).unwrap();

    let request = Request::builder()
        .method("GET")
        .uri("/api/auth/me")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_data["email"], "test@example.com");
    assert_eq!(response_data["tier"], 2);
    assert_eq!(response_data["kyc_status"], "approved");
}

#[tokio::test]
async fn test_unauthorized_access() {
    let (app, _db) = setup_test_app().await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/auth/me")
        .body(Body::empty())
        .unwrap();

    let response = app.router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_invalid_token() {
    let (app, _db) = setup_test_app().await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/auth/me")
        .header("authorization", "Bearer invalid_token")
        .body(Body::empty())
        .unwrap();

    let response = app.router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_password_validation() {
    let (app, _db) = setup_test_app().await;

    // Test weak password
    let register_request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "123".to_string(), // Too short
        stellar_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
    };

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&register_request).unwrap()))
        .unwrap();

    let response = app.router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response_data["error"].as_str().unwrap().contains("password"));
}

#[tokio::test]
async fn test_stellar_address_validation() {
    let (app, _db) = setup_test_app().await;

    // Test invalid Stellar address
    let register_request = RegisterRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        stellar_address: "invalid_address".to_string(),
    };

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&register_request).unwrap()))
        .unwrap();

    let response = app.router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_data: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(response_data["error"].as_str().unwrap().contains("Stellar address"));
}