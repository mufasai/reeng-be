mod handlers;
mod models;
mod state;

use axum::{
    extract::Json,
    http::Method,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use handlers::{auth, project};
use state::AppState;

// ==================== HEALTH CHECK ====================

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "message": "Server is running",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// ==================== MAIN ====================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment
    dotenv::dotenv().ok();

    // Create shared state with database connection
    let state = Arc::new(AppState::new().await?);

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/auth/login", post(auth::login))
        .route("/api/projects", post(project::create_project))
        .route("/api/projects", get(project::list_projects))
        .with_state(state)
        .layer(cors);

    // Start server
    let addr = "0.0.0.0:3000";
    println!("🚀 Server starting on http://{}", addr);
    println!("📝 Available endpoints:");
    println!("  GET    /api/health");
    println!("  POST   /api/auth/login");
    println!("  POST   /api/projects");
    println!("  GET    /api/projects");
    println!("\n✅ Login credentials for testing:");
    println!("   Email: admin@smartelco.com");
    println!("   Password: admin123");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
