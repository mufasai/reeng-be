use axum::{extract::{Json, Path}, http::StatusCode};
use std::sync::Arc;

use crate::models::{ApiResponse, CreateProjectRequest, Project, UpdateProjectRequest};
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

pub async fn delete_project(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Delete project from SurrealDB
    let query = "DELETE type::thing('projects', $id)";

    let _result = state
        .db
        .query(query)
        .bind(("id", project_id.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(format!("Project {} deleted successfully", project_id)),
        message: Some("Project deleted successfully".to_string()),
    }))
}

pub async fn get_project(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<Project>>, StatusCode> {
    // Get single project by ID from SurrealDB
    let query = "SELECT * FROM type::thing('projects', $id)";

    let mut response = state
        .db
        .query(query)
        .bind(("id", project_id.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let projects: Vec<Project> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Check if project exists
    if projects.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(projects[0].clone()),
        message: None,
    }))
}

pub async fn update_project(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Path(project_id): Path<String>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<ApiResponse<Project>>, StatusCode> {
    // First, get the existing project
    let query = "SELECT * FROM type::thing('projects', $id)";
    let mut response = state
        .db
        .query(query)
        .bind(("id", project_id.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let projects: Vec<Project> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if projects.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut existing_project = projects[0].clone();

    // Update only provided fields
    if let Some(name) = req.name {
        existing_project.name = name;
    }
    if let Some(lokasi) = req.lokasi {
        existing_project.lokasi = lokasi;
    }
    if let Some(value) = req.value {
        existing_project.value = value;
    }
    if let Some(cost) = req.cost {
        existing_project.cost = cost;
    }
    if let Some(tipe) = req.tipe {
        existing_project.tipe = tipe;
    }
    if let Some(keterangan) = req.keterangan {
        existing_project.keterangan = keterangan;
    }
    if let Some(tgi_start) = req.tgi_start {
        existing_project.tgi_start = Some(tgi_start);
    }
    if let Some(tgi_end) = req.tgi_end {
        existing_project.tgi_end = Some(tgi_end);
    }
    if let Some(status) = req.status {
        existing_project.status = Some(status);
    }

    // Update project in SurrealDB
    let update_query = r#"
        UPDATE type::thing('projects', $id) SET
            name = $name,
            lokasi = $lokasi,
            value = $value,
            cost = $cost,
            tipe = $tipe,
            keterangan = $keterangan,
            tgi_start = $tgi_start,
            tgi_end = $tgi_end,
            status = $status,
            updated_at = time::now()
    "#;

    let mut update_response = state
        .db
        .query(update_query)
        .bind(("id", project_id.clone()))
        .bind(("name", existing_project.name.clone()))
        .bind(("lokasi", existing_project.lokasi.clone()))
        .bind(("value", existing_project.value))
        .bind(("cost", existing_project.cost))
        .bind(("tipe", existing_project.tipe.clone()))
        .bind(("keterangan", existing_project.keterangan.clone()))
        .bind(("tgi_start", existing_project.tgi_start.clone()))
        .bind(("tgi_end", existing_project.tgi_end.clone()))
        .bind(("status", existing_project.status.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let updated_projects: Vec<Project> = update_response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let updated_project = updated_projects.first().cloned().unwrap_or(existing_project);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(updated_project),
        message: Some("Project updated successfully".to_string()),
    }))
}
