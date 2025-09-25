use loco_rs::prelude::*;
use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: HashSet<String>,
    pub allowed_methods: HashSet<Method>,
    pub allowed_headers: HashSet<String>,
    pub exposed_headers: HashSet<String>,
    pub allow_credentials: bool,
    pub max_age: Option<u64>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        let mut allowed_methods = HashSet::new();
        allowed_methods.insert(Method::GET);
        allowed_methods.insert(Method::POST);
        allowed_methods.insert(Method::PUT);
        allowed_methods.insert(Method::DELETE);
        allowed_methods.insert(Method::OPTIONS);

        let mut allowed_headers = HashSet::new();
        allowed_headers.insert("content-type".to_string());
        allowed_headers.insert("authorization".to_string());
        allowed_headers.insert("x-requested-with".to_string());

        let mut allowed_origins = HashSet::new();
        allowed_origins.insert("http://localhost:3000".to_string());

        Self {
            allowed_origins,
            allowed_methods,
            allowed_headers,
            exposed_headers: HashSet::new(),
            allow_credentials: true,
            max_age: Some(86400), // 24 hours
        }
    }
}

/// CORS middleware with security-focused configuration
pub async fn cors_middleware(
    State(cors_config): State<CorsConfig>,
    request: Request,
    next: Next,
) -> Result<Response> {
    let method = request.method().clone();
    
    // Get origin from request
    let origin = request.headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    // Check if origin is allowed
    let origin_allowed = cors_config.allowed_origins.contains(&origin) 
        || cors_config.allowed_origins.contains("*");

    // Handle preflight requests
    if method == Method::OPTIONS {
        return handle_preflight_request(&origin, &cors_config, request.headers());
    }

    // Process the actual request
    let mut response = next.run(request).await;

    // Add CORS headers to response
    if origin_allowed {
        add_cors_headers(&mut response, &origin, &cors_config);
    }

    Ok(response)
}

/// Handle CORS preflight requests
fn handle_preflight_request(
    origin: &str,
    cors_config: &CorsConfig,
    request_headers: &HeaderMap,
) -> Result<Response> {
    // Check if origin is allowed
    if !cors_config.allowed_origins.contains(origin) && !cors_config.allowed_origins.contains("*") {
        return Err(Error::string("Origin not allowed"));
    }

    // Check requested method
    let requested_method = request_headers
        .get("access-control-request-method")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<Method>().ok());

    if let Some(method) = requested_method {
        if !cors_config.allowed_methods.contains(&method) {
            return Err(Error::string("Method not allowed"));
        }
    }

    // Check requested headers
    if let Some(requested_headers) = request_headers.get("access-control-request-headers") {
        if let Ok(headers_str) = requested_headers.to_str() {
            let headers: Vec<String> = headers_str.split(',').map(|h| h.trim().to_lowercase()).collect();
            for header in headers {
                if !cors_config.allowed_headers.contains(&header) {
                    return Err(Error::string("Header not allowed"));
                }
            }
        }
    }

    // Create preflight response
    let mut response = Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(axum::body::Body::empty())
        .unwrap();

    add_cors_headers(&mut response, origin, cors_config);

    // Add preflight-specific headers
    let headers = response.headers_mut();
    
    if let Some(max_age) = cors_config.max_age {
        headers.insert(
            "access-control-max-age",
            HeaderValue::from_str(&max_age.to_string()).unwrap(),
        );
    }

    // Add allowed methods
    let methods: Vec<String> = cors_config
        .allowed_methods
        .iter()
        .map(|m| m.to_string())
        .collect();
    headers.insert(
        "access-control-allow-methods",
        HeaderValue::from_str(&methods.join(", ")).unwrap(),
    );

    // Add allowed headers
    let allowed_headers: Vec<String> = cors_config.allowed_headers.iter().cloned().collect();
    headers.insert(
        "access-control-allow-headers",
        HeaderValue::from_str(&allowed_headers.join(", ")).unwrap(),
    );

    Ok(response)
}

/// Add CORS headers to response
fn add_cors_headers(response: &mut Response, origin: &str, cors_config: &CorsConfig) {
    let headers = response.headers_mut();

    // Add origin header
    if !origin.is_empty() {
        headers.insert(
            "access-control-allow-origin",
            HeaderValue::from_str(origin).unwrap(),
        );
    }

    // Add credentials header
    if cors_config.allow_credentials {
        headers.insert(
            "access-control-allow-credentials",
            HeaderValue::from_static("true"),
        );
    }

    // Add exposed headers
    if !cors_config.exposed_headers.is_empty() {
        let exposed: Vec<String> = cors_config.exposed_headers.iter().cloned().collect();
        headers.insert(
            "access-control-expose-headers",
            HeaderValue::from_str(&exposed.join(", ")).unwrap(),
        );
    }

    // Add Vary header for proper caching
    headers.insert("vary", HeaderValue::from_static("Origin"));
}

/// Create CORS configuration from environment or config file
pub fn create_cors_config(allowed_origins: Vec<String>) -> CorsConfig {
    let mut config = CorsConfig::default();
    
    // Override with provided origins
    config.allowed_origins = allowed_origins.into_iter().collect();
    
    // Add security-focused headers
    config.exposed_headers.insert("x-total-count".to_string());
    config.exposed_headers.insert("x-pagination".to_string());
    
    config
}