use axum::{extract::Json, http::StatusCode};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{ApiResponse, CreateProjectRequest, Project};
use crate::state::AppState;

pub async fn create_project(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<ApiResponse<Project>>, StatusCode> {
    // Create project with generated ID
    let project = Project {
        id: format!("projects:{}", Uuid::new_v4().to_string()),
        project_name: req.project_name,
        lokasi: req.lokasi,
        budget: req.budget,
        tipe: req.tipe,
        keterangan: req.keterangan,
        project_document: req.project_document,
        sites: req.sites.into_iter().map(|mut site| {
            site.id = Some(format!("site:{}", Uuid::new_v4().to_string()));
            site
        }).collect(),
        created_at: None,
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
