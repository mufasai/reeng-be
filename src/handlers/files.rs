use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::models::{ApiResponse, ProjectFile, SiteFile, CreateProjectFileRequest, CreateSiteFileRequest};
use crate::state::AppState;

// ==================== PROJECT FILE HANDLERS ====================

pub async fn create_project_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateProjectFileRequest>,
) -> Result<Json<ApiResponse<ProjectFile>>, StatusCode> {
    let query = r#"
        CREATE project_files CONTENT {
            project_id: type::thing($project_id),
            title: $title,
            filename: $filename,
            original_name: $original_name,
            bucket: $bucket,
            key: $key,
            mime_type: $mime_type,
            size: $size,
            disk: $disk,
            visibility: $visibility,
            uploaded_at: time::now(),
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let mut result = state.db.query(query)
        .bind(("project_id", req.project_id.clone()))
        .bind(("title", req.title.clone()))
        .bind(("filename", req.filename.clone()))
        .bind(("original_name", req.original_name.clone()))
        .bind(("bucket", req.bucket.clone()))
        .bind(("key", req.key.clone()))
        .bind(("mime_type", req.mime_type.clone()))
        .bind(("size", req.size))
        .bind(("disk", req.disk.clone()))
        .bind(("visibility", req.visibility.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file: Option<ProjectFile> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match file {
        Some(file) => Ok(Json(ApiResponse {
            success: true,
            data: Some(file),
            message: Some("Project file uploaded successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_project_files(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<ProjectFile>>>, StatusCode> {
    let query = "SELECT * FROM project_files WHERE project_id = type::thing('projects', $id) ORDER BY uploaded_at DESC";

    let mut result = state.db.query(query)
        .bind(("id", project_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let files: Vec<ProjectFile> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(files),
        message: None,
    }))
}

pub async fn delete_project_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let query = "DELETE type::thing('project_files', $id)";

    let _result = state.db.query(query)
        .bind(("id", file_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some("File deleted successfully".to_string()),
        message: None,
    }))
}

// ==================== SITE FILE HANDLERS ====================

pub async fn create_site_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSiteFileRequest>,
) -> Result<Json<ApiResponse<SiteFile>>, StatusCode> {
    let query = r#"
        CREATE site_files CONTENT {
            site_id: type::thing($site_id),
            title: $title,
            filename: $filename,
            original_name: $original_name,
            bucket: $bucket,
            key: $key,
            mime_type: $mime_type,
            size: $size,
            disk: $disk,
            visibility: $visibility,
            uploaded_at: time::now(),
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let mut result = state.db.query(query)
        .bind(("site_id", req.site_id.clone()))
        .bind(("title", req.title.clone()))
        .bind(("filename", req.filename.clone()))
        .bind(("original_name", req.original_name.clone()))
        .bind(("bucket", req.bucket.clone()))
        .bind(("key", req.key.clone()))
        .bind(("mime_type", req.mime_type.clone()))
        .bind(("size", req.size))
        .bind(("disk", req.disk.clone()))
        .bind(("visibility", req.visibility.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file: Option<SiteFile> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match file {
        Some(file) => Ok(Json(ApiResponse {
            success: true,
            data: Some(file),
            message: Some("Site file uploaded successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_site_files(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<SiteFile>>>, StatusCode> {
    let query = "SELECT * FROM site_files WHERE site_id = type::thing('sites', $id) ORDER BY uploaded_at DESC";

    let mut result = state.db.query(query)
        .bind(("id", site_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let files: Vec<SiteFile> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(files),
        message: None,
    }))
}

pub async fn delete_site_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let query = "DELETE type::thing('site_files', $id)";

    let _result = state.db.query(query)
        .bind(("id", file_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some("File deleted successfully".to_string()),
        message: None,
    }))
}
