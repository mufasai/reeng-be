use axum::{extract::Json, http::StatusCode};
use std::sync::Arc;

use crate::models::{ApiResponse, CreateProjectRequest, Project};
use crate::state::AppState;

pub async fn create_project(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<ApiResponse<Project>>, StatusCode> {
    // Create project with new structure
    let project = Project {
        id: None,
        name: req.name,  // Changed from project_name
        lokasi: req.lokasi,
        value: req.value,  // Changed from budget
        cost: req.cost.unwrap_or(0),  // NEW field
        keterangan: req.keterangan,
        tipe: req.tipe,
        tgi_start: req.tgi_start,  // NEW
        tgi_end: req.tgi_end,      // NEW
        status: req.status.or(Some("active".to_string())),  // NEW with default
        created_at: None,
        updated_at: None,
    };

    // Save to SurrealDB
    let created: Option<Project> = state
        .db
        .create("projects")
        .content(project.clone())
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let created_project = created.unwrap_or(project);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(created_project),
        message: Some("Project created successfully".to_string()),
    }))
}

pub async fn list_projects(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Project>>>, StatusCode> {
    // Query all projects from SurrealDB
    let mut response = state
        .db
        .query("SELECT * FROM projects ORDER BY created_at DESC")
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let projects: Vec<Project> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(projects),
        message: None,
    }))
}
