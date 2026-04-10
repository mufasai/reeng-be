use axum::{
    extract::{Path, State, Multipart},
    http::{StatusCode, header, HeaderMap, HeaderValue},
    response::Response,
    body::Body,
    Json,
};
use std::sync::Arc;
use surrealdb::sql::Thing;
use base64::Engine;
use crate::models::{ApiResponse, ProjectFile, SiteFile, SiteEvidence, CreateProjectFileRequest, CreateSiteFileRequest};
use crate::state::AppState;
use crate::common::parse_thing_id;

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
    let site_thing = crate::common::parse_thing_id(&site_id, "sites")?;

    let query = "SELECT * FROM site_files WHERE site_id = $site_id ORDER BY uploaded_at DESC";

    let mut result = state.db.query(query)
        .bind(("site_id", site_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error listing site files: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let files: Vec<SiteFile> = result.take(0).map_err(|e| {
        eprintln!("Parse error listing site files: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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

// ==================== MULTIPART FILE UPLOAD HANDLERS ====================

pub async fn upload_project_file_multipart(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<ProjectFile>>, StatusCode> {
    let mut title: Option<String> = None;
    let mut category: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_content_type: Option<String> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                file_content_type = field.content_type().map(|s| s.to_string());
                let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                file_data = Some(bytes.to_vec());
            }
            "title" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                title = Some(text);
            }
            "category" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                category = Some(text);
            }
            _ => {}
        }
    }

    // Validate required fields
    let file_bytes = file_data.ok_or(StatusCode::BAD_REQUEST)?;
    let filename = file_name.ok_or(StatusCode::BAD_REQUEST)?;
    let title_str = title.unwrap_or_else(|| filename.clone());
    let mime_type = file_content_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let file_size = file_bytes.len() as i64;

    // Convert to base64 data URL
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_bytes);
    let data_url = format!("data:{};base64,{}", mime_type, base64_data);

    // Save to database
    let query = r#"
        CREATE project_files CONTENT {
            project_id: type::thing('projects', $project_id),
            title: $title,
            filename: $filename,
            original_name: $filename,
            file_data: $file_data,
            key: $filename,
            mime_type: $mime_type,
            size: $size,
            uploaded_at: time::now(),
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let mut result = state.db.query(query)
        .bind(("project_id", project_id))
        .bind(("title", title_str))
        .bind(("filename", filename))
        .bind(("file_data", data_url))
        .bind(("mime_type", mime_type))
        .bind(("size", file_size))
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

pub async fn upload_site_file_multipart(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<SiteFile>>, StatusCode> {
    let mut title: Option<String> = None;
    let mut category: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_content_type: Option<String> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        eprintln!("Multipart parsing error: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        let name = field.name().unwrap_or("").to_string();
        eprintln!("Processing field: {}", name);
        
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                file_content_type = field.content_type().map(|s| s.to_string());
                let bytes = field.bytes().await.map_err(|e| {
                    eprintln!("Error reading field bytes: {}", e);
                    StatusCode::BAD_REQUEST
                })?;
                file_data = Some(bytes.to_vec());
                eprintln!("File field detected: {}, size: {} bytes", file_name.as_deref().unwrap_or("unknown"), bytes.len());
            }
            "title" => {
                let text = field.text().await.map_err(|e| {
                    eprintln!("Error reading title text: {}", e);
                    StatusCode::BAD_REQUEST
                })?;
                title = Some(text.clone());
                eprintln!("Title field detected: {}", text);
            }
            "category" => {
                let text = field.text().await.map_err(|e| {
                    eprintln!("Error reading category text: {}", e);
                    StatusCode::BAD_REQUEST
                })?;
                category = Some(text.clone());
                eprintln!("Category field detected: {}", text);
            }
            _ => {
                eprintln!("Unknown field detected: {}", name);
            }
        }
    }

    // Validate required fields
    let file_bytes = file_data.ok_or(StatusCode::BAD_REQUEST)?;
    let filename = file_name.ok_or(StatusCode::BAD_REQUEST)?;
    let title_str = title.unwrap_or_else(|| filename.clone());
    let mime_type = file_content_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let file_size = file_bytes.len() as i64;

    // Convert to base64 data URL
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_bytes);
    let data_url = format!("data:{};base64,{}", mime_type, base64_data);

    // Save to database
    let query = r#"
        CREATE site_files CONTENT {
            site_id: type::thing('sites', $site_id),
            title: $title,
            filename: $filename,
            original_name: $filename,
            file_data: $file_data,
            key: $filename,
            mime_type: $mime_type,
            size: $size,
            uploaded_at: time::now(),
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let mut result = state.db.query(query)
        .bind(("site_id", site_id))
        .bind(("title", title_str))
        .bind(("filename", filename))
        .bind(("file_data", data_url))
        .bind(("mime_type", mime_type))
        .bind(("size", file_size))
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

pub async fn download_project_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    let thing = Thing::try_from(("project_files", file_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = "SELECT * FROM $file_id";
    let mut result = state.db.query(query)
        .bind(("file_id", thing))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file: Option<ProjectFile> = result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match file {
        Some(file) => {
            let data_url = file.file_data.ok_or(StatusCode::NOT_FOUND)?;
            let filename = file.filename;
            let mime_type = file.mime_type;

            // Parse data URL
            let parts: Vec<&str> = data_url.split(',').collect();
            if parts.len() != 2 {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            let base64_data = parts[1];
            let file_bytes = base64::engine::general_purpose::STANDARD
                .decode(base64_data)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Build response with headers
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&mime_type)
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("inline; filename=\"{}\"", filename))
                    .unwrap_or(HeaderValue::from_static("inline")),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&file_bytes.len().to_string()).unwrap(),
            );

            let body = Body::from(file_bytes);
            let mut response = Response::new(body);
            *response.headers_mut() = headers;
            *response.status_mut() = StatusCode::OK;

            Ok(response)
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn download_site_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    let thing = Thing::try_from(("site_files", file_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = "SELECT * FROM $file_id";
    let mut result = state.db.query(query)
        .bind(("file_id", thing))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file: Option<SiteFile> = result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match file {
        Some(file) => {
            let data_url = file.file_data.ok_or(StatusCode::NOT_FOUND)?;
            let filename = file.filename;
            let mime_type = file.mime_type;

            // Parse data URL
            let parts: Vec<&str> = data_url.split(',').collect();
            if parts.len() != 2 {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            let base64_data = parts[1];
            let file_bytes = base64::engine::general_purpose::STANDARD
                .decode(base64_data)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Build response with headers
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&mime_type)
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("inline; filename=\"{}\"", filename))
                    .unwrap_or(HeaderValue::from_static("inline")),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&file_bytes.len().to_string()).unwrap(),
            );

            let body = Body::from(file_bytes);
            let mut response = Response::new(body);
            *response.headers_mut() = headers;
            *response.status_mut() = StatusCode::OK;

            Ok(response)
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn download_site_evidence(
    State(state): State<Arc<AppState>>,
    Path(evidence_id): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    let thing = parse_thing_id(&evidence_id, "site_evidence").map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut response = state.db.query("SELECT * FROM site_evidence WHERE id = $id")
        .bind(("id", thing))
        .await
        .map_err(|e| {
            eprintln!("Database error selecting site evidence: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let evidence: Option<SiteEvidence> = response.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match evidence {
        Some(evidence) => {
            // Use file_data if available, otherwise fallback to url (previously file_url)
            let data_url = evidence.file_data
                .or(evidence.url)
                .ok_or(StatusCode::NOT_FOUND)?;
            
            let filename = evidence.filename;
            let mime_type = evidence.mime_type.unwrap_or_else(|| "image/jpeg".to_string());

            // Parse data URL (format: data:image/jpeg;base64,...)
            let parts: Vec<&str> = data_url.split(',').collect();
            if parts.len() != 2 {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            let base64_data = parts[1];
            let file_bytes = base64::engine::general_purpose::STANDARD
                .decode(base64_data)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Build response with headers
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&mime_type)
                    .unwrap_or(HeaderValue::from_static("image/jpeg")),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("inline; filename=\"{}\"", filename))
                    .unwrap_or(HeaderValue::from_static("inline")),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&file_bytes.len().to_string()).unwrap(),
            );

            let body = Body::from(file_bytes);
            let mut response = Response::new(body);
            *response.headers_mut() = headers;
            *response.status_mut() = StatusCode::OK;

            Ok(response)
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

