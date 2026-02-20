use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::models::{ApiResponse, Termin, TerminFile, CreateTerminRequest, CreateTerminFileRequest};
use crate::state::AppState;

// ==================== TERMIN HANDLERS ====================

pub async fn create_termin(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTerminRequest>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let query = r#"
        CREATE termins CONTENT {
            project_id: type::thing($project_id),
            site_id: type::thing($site_id),
            type_termin: $type_termin,
            tgl_terima: $tgl_terima,
            jumlah: $jumlah,
            status: $status,
            keterangan: $keterangan,
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let status = req.status.unwrap_or_else(|| "pending".to_string());

    let mut result = state.db.query(query)
        .bind(("project_id", req.project_id.clone()))
        .bind(("site_id", req.site_id.clone()))
        .bind(("type_termin", req.type_termin.clone()))
        .bind(("tgl_terima", req.tgl_terima.clone()))
        .bind(("jumlah", req.jumlah))
        .bind(("status", status))
        .bind(("keterangan", req.keterangan.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match termin {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: Some("Termin created successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_termins(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Termin>>>, StatusCode> {
    let query = "SELECT * FROM termins ORDER BY created_at DESC";

    let mut result = state.db.query(query)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termins: Vec<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(termins),
        message: None,
    }))
}

pub async fn get_termins_by_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Termin>>>, StatusCode> {
    let query = "SELECT * FROM termins WHERE project_id = type::thing('projects', $id) ORDER BY created_at DESC";

    let mut result = state.db.query(query)
        .bind(("id", project_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termins: Vec<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(termins),
        message: None,
    }))
}

pub async fn get_termins_by_site(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Termin>>>, StatusCode> {
    let query = "SELECT * FROM termins WHERE site_id = type::thing('sites', $id) ORDER BY created_at DESC";

    let mut result = state.db.query(query)
        .bind(("id", site_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termins: Vec<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(termins),
        message: None,
    }))
}

// ==================== TERMIN FILE HANDLERS ====================

pub async fn create_termin_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTerminFileRequest>,
) -> Result<Json<ApiResponse<TerminFile>>, StatusCode> {
    let query = r#"
        CREATE termin_files CONTENT {
            termin_id: type::thing($termin_id),
            category: $category,
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
        .bind(("termin_id", req.termin_id.clone()))
        .bind(("category", req.category.clone()))
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

    let file: Option<TerminFile> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match file {
        Some(file) => Ok(Json(ApiResponse {
            success: true,
            data: Some(file),
            message: Some("Termin file uploaded successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_termin_files(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<TerminFile>>>, StatusCode> {
    let query = "SELECT * FROM termin_files WHERE termin_id = type::thing('termins', $id) ORDER BY uploaded_at DESC";

    let mut result = state.db.query(query)
        .bind(("id", termin_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let files: Vec<TerminFile> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(files),
        message: None,
    }))
}

pub async fn delete_termin_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let query = "DELETE type::thing('termin_files', $id)";

    let _result = state.db.query(query)
        .bind(("id", file_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some("Termin file deleted successfully".to_string()),
        message: None,
    }))
}
