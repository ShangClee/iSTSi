use loco_rs::prelude::*;
use axum::{
    extract::{Request, State},
    http::{header, HeaderMap},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user ID)
    pub exp: usize,       // Expiration time
    pub iat: usize,       // Issued at
    pub jti: String,      // JWT ID for token revocation
    pub role: String,     // User role for authorization
    pub permissions: Vec<String>, // Specific permissions
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiration: u64,
    pub issuer: String,
    pub audience: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "default-secret-change-in-production".to_string(),
            token_expiration: 86400, // 24 hours
            issuer: "bitcoin-custody-backend".to_string(),
            audience: "bitcoin-custody-frontend".to_string(),
        }
    }
}

/// JWT Authentication middleware
pub async fn jwt_auth_middleware(
    State(auth_config): State<AuthConfig>,
    mut request: Request,
    next: Next,
) -> Result<Response> {
    let headers = request.headers();
    
    // Extract token from Authorization header
    let token = extract_token_from_headers(headers)
        .ok_or_else(|| Error::Unauthorized("Missing or invalid authorization header".to_string()))?;
    
    // Validate JWT token
    let claims = validate_jwt_token(&token, &auth_config.jwt_secret)?;
    
    // Check token expiration
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    
    if claims.exp < current_time {
        return Err(Error::Unauthorized("Token expired".to_string()));
    }
    
    // Add claims to request extensions for use in handlers
    request.extensions_mut().insert(claims);
    
    Ok(next.run(request).await)
}

/// Extract Bearer token from Authorization header
fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get(header::AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;
    
    if auth_str.starts_with("Bearer ") {
        Some(auth_str[7..].to_string())
    } else {
        None
    }
}

/// Validate JWT token and extract claims
pub fn validate_jwt_token(token: &str, secret: &str) -> Result<Claims> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.validate_nbf = true;
    
    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| Error::string(&format!("JWT validation failed: {}", e)))?;
    
    Ok(token_data.claims)
}

/// Generate JWT token for authenticated user
pub fn generate_jwt_token(
    user_id: &str,
    role: &str,
    permissions: Vec<String>,
    config: &AuthConfig,
) -> Result<String> {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    
    let claims = Claims {
        sub: user_id.to_string(),
        exp: current_time + config.token_expiration as usize,
        iat: current_time,
        jti: Uuid::new_v4().to_string(),
        role: role.to_string(),
        permissions,
    };
    
    let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_ref());
    let header = Header::default();
    
    encode(&header, &claims, &encoding_key)
        .map_err(|e| Error::string(&format!("JWT generation failed: {}", e)))
}

/// Role-based authorization middleware
pub fn require_role(required_role: String) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let required_role = required_role.clone();
        Box::pin(async move {
            let claims = request
                .extensions()
                .get::<Claims>()
                .ok_or_else(|| Error::Unauthorized("Missing authentication claims".to_string()))?;
            
            if claims.role != required_role && claims.role != "admin" {
                return Err(Error::string("Insufficient role permissions"));
            }
            
            Ok(next.run(request).await)
        })
    }
}

/// Permission-based authorization middleware
pub fn require_permission(required_permission: String) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let required_permission = required_permission.clone();
        Box::pin(async move {
            let claims = request
                .extensions()
                .get::<Claims>()
                .ok_or_else(|| Error::Unauthorized("Missing authentication claims".to_string()))?;
            
            if !claims.permissions.contains(&required_permission) && claims.role != "admin" {
                return Err(Error::string("Insufficient permissions"));
            }
            
            Ok(next.run(request).await)
        })
    }
}

/// Rate limiting middleware to prevent abuse
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response> {
    // TODO: Implement proper rate limiting with Redis or in-memory store
    // For now, this is a placeholder that allows all requests
    Ok(next.run(request).await)
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Add security headers
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    headers.insert("Content-Security-Policy", 
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".parse().unwrap());
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    
    response
}