mod handlers;
mod models;
mod state;

use axum::{
    extract::Json,
    http::Method,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use handlers::{auth, project, site, people, costs, materials, regions, files, termins};
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
        // Auth routes
        .route("/api/auth/login", post(auth::login))
        // Project routes
        .route("/api/projects", post(project::create_project))
        .route("/api/projects", get(project::list_projects))
        .route("/api/projects/:id", get(project::get_project))
        .route("/api/projects/:id", put(project::update_project))
        .route("/api/projects/:id", delete(project::delete_project))
        // Site routes
        .route("/api/sites", post(site::create_site))
        .route("/api/sites", get(site::list_sites))
        .route("/api/sites/project/:project_id", get(site::get_sites_by_project))
        .route("/api/sites/:id", put(site::update_site))
        .route("/api/sites/:id", delete(site::delete_site))
        // People routes
        .route("/api/people", post(people::create_people))
        .route("/api/people", get(people::list_people))
        .route("/api/people/:id", get(people::get_people))
        .route("/api/people/:id", put(people::update_people))
        .route("/api/people/:id", delete(people::delete_people))
        // Cost routes
        .route("/api/costs", post(costs::create_cost))
        .route("/api/costs", get(costs::list_costs))
        .route("/api/costs/project/:project_id", get(costs::get_costs_by_project))
        .route("/api/costs/site/:site_id", get(costs::get_costs_by_site))
        .route("/api/costs/:cost_id/approve", post(costs::approve_cost))
        // Material routes
        .route("/api/materials", post(materials::create_material))
        .route("/api/materials", get(materials::list_materials))
        .route("/api/materials/project/:project_id", get(materials::get_materials_by_project))
        .route("/api/materials/site/:site_id", get(materials::get_materials_by_site))
        // Area & Region routes
        .route("/api/areas", post(regions::create_area))
        .route("/api/areas", get(regions::list_areas))
        .route("/api/regions", post(regions::create_region))
        .route("/api/regions", get(regions::list_regions))
        .route("/api/regions/area/:area_id", get(regions::get_regions_by_area))
        // File routes
        .route("/api/project-files", post(files::create_project_file))
        .route("/api/projects/:project_id/files", get(files::list_project_files))
        .route("/api/project-files/:file_id/delete", axum::routing::delete(files::delete_project_file))
        .route("/api/site-files", post(files::create_site_file))
        .route("/api/sites/:site_id/files", get(files::list_site_files))
        .route("/api/site-files/:file_id/delete", axum::routing::delete(files::delete_site_file))
        // Termin routes
        .route("/api/termins", post(termins::create_termin))
        .route("/api/termins", get(termins::list_termins))
        .route("/api/termins/project/:project_id", get(termins::get_termins_by_project))
        .route("/api/termins/site/:site_id", get(termins::get_termins_by_site))
        .route("/api/termins/:termin_id", get(termins::get_termin_by_id))
        .route("/api/termins/:termin_id", put(termins::update_termin))
        .route("/api/termins/:termin_id", axum::routing::delete(termins::delete_termin))
        .route("/api/termins/:termin_id/submit", post(termins::submit_termin))
        .route("/api/termins/:termin_id/review", post(termins::review_termin))
        .route("/api/termins/:termin_id/approve", post(termins::approve_termin))
        .route("/api/termins/:termin_id/pay", post(termins::pay_termin))
        .route("/api/termin-files", post(termins::create_termin_file))
        .route("/api/termins/:termin_id/files", get(termins::list_termin_files))
        .route("/api/termin-files/:file_id/delete", axum::routing::delete(termins::delete_termin_file))
        .with_state(state)
        .layer(cors);

    // Start server
    let addr = "0.0.0.0:3000";
    println!("🚀 Server starting on http://{}", addr);
    println!("📝 Available endpoints:");
    println!("  GET    /api/health");
    println!("\n🔐 Auth:");
    println!("  POST   /api/auth/login");
    println!("\n📊 Projects:");
    println!("  POST   /api/projects");
    println!("  GET    /api/projects");
    println!("  GET    /api/projects/:id");
    println!("  PUT    /api/projects/:id");
    println!("  DELETE /api/projects/:id");
    println!("\n🏗️  Sites:");
    println!("  POST   /api/sites");
    println!("  GET    /api/sites");
    println!("  GET    /api/sites/project/:project_id");
    println!("\n👥 People:");
    println!("  POST   /api/people");
    println!("  GET    /api/people");
    println!("  GET    /api/people/:id");
    println!("  PUT    /api/people/:id");
    println!("  DELETE /api/people/:id");
    println!("\n💰 Costs:");
    println!("  POST   /api/costs");
    println!("  GET    /api/costs");
    println!("  GET    /api/costs/project/:project_id");
    println!("  GET    /api/costs/site/:site_id");
    println!("  POST   /api/costs/:cost_id/approve");
    println!("\n📦 Materials:");
    println!("  POST   /api/materials");
    println!("  GET    /api/materials");
    println!("  GET    /api/materials/project/:project_id");
    println!("  GET    /api/materials/site/:site_id");
    println!("\n🌍 Areas & Regions:");
    println!("  POST   /api/areas");
    println!("  GET    /api/areas");
    println!("  POST   /api/regions");
    println!("  GET    /api/regions");
    println!("  GET    /api/regions/area/:area_id");
    println!("\n📁 Files:");
    println!("  POST   /api/project-files");
    println!("  GET    /api/projects/:project_id/files");
    println!("  DELETE /api/project-files/:file_id/delete");
    println!("  POST   /api/site-files");
    println!("  GET    /api/sites/:site_id/files");
    println!("  DELETE /api/site-files/:file_id/delete");
    println!("\n💵 Termins:");
    println!("  POST   /api/termins");
    println!("  GET    /api/termins");
    println!("  GET    /api/termins/project/:project_id");
    println!("  GET    /api/termins/site/:site_id");
    println!("  GET    /api/termins/:termin_id");
    println!("  PUT    /api/termins/:termin_id");
    println!("  DELETE /api/termins/:termin_id");
    println!("  POST   /api/termins/:termin_id/submit");
    println!("  POST   /api/termins/:termin_id/review");
    println!("  POST   /api/termins/:termin_id/approve");
    println!("  POST   /api/termins/:termin_id/pay");
    println!("  POST   /api/termin-files");
    println!("  GET    /api/termins/:termin_id/files");
    println!("  DELETE /api/termin-files/:file_id/delete");
    println!("\n✅ Login credentials for testing:");
    println!("   Email: admin@smartelco.com");
    println!("   Password: admin123");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
