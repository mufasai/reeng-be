use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use surrealdb::sql::Thing;
use crate::models::{
    ApiResponse, Site, Termin, TerminFile, CreateTerminRequest, UpdateTerminRequest,
    CreateTerminFileRequest, SubmitTerminRequest, ReviewTerminRequest, ApproveTerminRequest, PayTerminRequest,
};
use crate::state::AppState;

// ==================== TERMIN HANDLERS ====================

pub async fn create_termin(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTerminRequest>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    // Step 1: Fetch the site to get maximal_budget for validation
    let fetch_site_query = "SELECT * FROM type::thing($site_id)";
    let mut site_result = state.db.query(fetch_site_query)
        .bind(("site_id", req.site_id.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let site: Option<Site> = site_result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let site = site.ok_or(StatusCode::NOT_FOUND)?;
    
    // Step 2: Calculate expected amount based on percentage
    let expected_amount = (site.maximal_budget * req.percentage as i64) / 100;
    
    // Step 3: Validate that jumlah matches expected amount (allow 1% tolerance)
    let tolerance = expected_amount / 100; // 1% tolerance
    if (req.jumlah - expected_amount).abs() > tolerance {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some(format!(
                "Validation failed: jumlah ({}) does not match expected amount ({}) based on {}% of site maximal_budget ({})",
                req.jumlah, expected_amount, req.percentage, site.maximal_budget
            )),
        }));
    }
    
    // Step 4: Determine status and submit tracking based on submitted_by
    let (status, submitted_by, _submitted_at) = if let Some(submitter) = &req.submitted_by {
        ("pending_review".to_string(), Some(submitter.clone()), Some("time::now()"))
    } else {
        (req.status.clone().unwrap_or_else(|| "draft".to_string()), None, None)
    };
    
    // Step 5: Create the termin
    let query = if submitted_by.is_some() {
        r#"
        CREATE termins CONTENT {
            project_id: type::thing($project_id),
            site_id: type::thing($site_id),
            type_termin: $type_termin,
            tgl_terima: $tgl_terima,
            jumlah: $jumlah,
            termin_ke: $termin_ke,
            percentage: $percentage,
            status: $status,
            keterangan: $keterangan,
            submitted_by: $submitted_by,
            submitted_at: time::now(),
            reviewed_by: NONE,
            reviewed_at: NONE,
            catatan_review: NONE,
            approved_by: NONE,
            approved_at: NONE,
            catatan_approval: NONE,
            paid_by: NONE,
            paid_at: NONE,
            jumlah_dibayar: NONE,
            referensi_pembayaran: NONE,
            catatan_pembayaran: NONE,
            bukti_pembayaran: NONE,
            created_at: time::now(),
            updated_at: time::now()
        }
        "#
    } else {
        r#"
        CREATE termins CONTENT {
            project_id: type::thing($project_id),
            site_id: type::thing($site_id),
            type_termin: $type_termin,
            tgl_terima: $tgl_terima,
            jumlah: $jumlah,
            termin_ke: $termin_ke,
            percentage: $percentage,
            status: $status,
            keterangan: $keterangan,
            submitted_by: NONE,
            submitted_at: NONE,
            reviewed_by: NONE,
            reviewed_at: NONE,
            catatan_review: NONE,
            approved_by: NONE,
            approved_at: NONE,
            catatan_approval: NONE,
            paid_by: NONE,
            paid_at: NONE,
            jumlah_dibayar: NONE,
            referensi_pembayaran: NONE,
            catatan_pembayaran: NONE,
            bukti_pembayaran: NONE,
            created_at: time::now(),
            updated_at: time::now()
        }
        "#
    };

    let mut query_builder = state.db.query(query)
        .bind(("project_id", req.project_id.clone()))
        .bind(("site_id", req.site_id.clone()))
        .bind(("type_termin", req.type_termin.clone()))
        .bind(("tgl_terima", req.tgl_terima.clone()))
        .bind(("jumlah", req.jumlah))
        .bind(("termin_ke", req.termin_ke))
        .bind(("percentage", req.percentage))
        .bind(("status", status))
        .bind(("keterangan", req.keterangan.clone()));
    
    if let Some(submitter) = submitted_by {
        query_builder = query_builder.bind(("submitted_by", submitter));
    }
    
    let mut result = query_builder
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

pub async fn get_termin_by_id(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = "SELECT * FROM $termin_id";

    let mut result = state.db.query(query)
        .bind(("termin_id", thing))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match termin {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: None,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_termin(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
    Json(req): Json<UpdateTerminRequest>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Check if termin exists and is in draft status
    let check_query = "SELECT * FROM $termin_id";
    let mut check_result = state.db.query(check_query)
        .bind(("termin_id", thing.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let existing: Option<Termin> = check_result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if existing.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let existing = existing.unwrap();
    if existing.status != "draft" {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("Only draft termins can be updated".to_string()),
        }));
    }

    // Build dynamic update query
    let mut set_clauses = vec!["updated_at = time::now()"];
    let mut bindings = Vec::new();

    if req.type_termin.is_some() {
        set_clauses.push("type_termin = $type_termin");
        bindings.push(("type_termin", req.type_termin.clone()));
    }
    if req.tgl_terima.is_some() {
        set_clauses.push("tgl_terima = $tgl_terima");
        bindings.push(("tgl_terima", req.tgl_terima.clone()));
    }
    if req.jumlah.is_some() {
        set_clauses.push("jumlah = $jumlah");
    }
    if req.keterangan.is_some() {
        set_clauses.push("keterangan = $keterangan");
        bindings.push(("keterangan", req.keterangan.clone()));
    }

    let set_clause = set_clauses.join(", ");
    let query = format!("UPDATE $termin_id SET {}", set_clause);

    let mut db_query = state.db.query(&query)
        .bind(("termin_id", thing));

    for (key, value) in bindings {
        db_query = db_query.bind((key, value));
    }
    if let Some(jumlah) = req.jumlah {
        db_query = db_query.bind(("jumlah", jumlah));
    }

    let mut result = db_query.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let updated: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match updated {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: Some("Termin updated successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn submit_termin(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
    Json(req): Json<SubmitTerminRequest>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = r#"
        UPDATE $termin_id SET 
            status = 'pending_review',
            submitted_by = $submitted_by,
            submitted_at = time::now(),
            updated_at = time::now()
    "#;

    let mut result = state.db.query(query)
        .bind(("termin_id", thing))
        .bind(("submitted_by", req.submitter_name.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match termin {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: Some("Termin submitted for review".to_string()),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn review_termin(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
    Json(req): Json<ReviewTerminRequest>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let new_status = if req.approve { "reviewed" } else { "draft" };

    let query = r#"
        UPDATE $termin_id SET 
            status = $status,
            reviewed_by = $reviewed_by,
            reviewed_at = time::now(),
            catatan_review = $catatan_review,
            updated_at = time::now()
    "#;

    let mut result = state.db.query(query)
        .bind(("termin_id", thing))
        .bind(("status", new_status))
        .bind(("reviewed_by", req.reviewer_name.clone()))
        .bind(("catatan_review", req.catatan_review.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let message = if req.approve {
        "Termin reviewed and approved by Field Head. Waiting for Director approval.".to_string()
    } else {
        "Termin rejected by Field Head. Returned to draft.".to_string()
    };

    match termin {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: Some(message),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn approve_termin(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
    Json(req): Json<ApproveTerminRequest>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let new_status = if req.approve { "approved" } else { "draft" };

    let query = r#"
        UPDATE $termin_id SET 
            status = $status,
            approved_by = $approved_by,
            approved_at = time::now(),
            catatan_approval = $catatan_approval,
            updated_at = time::now()
    "#;

    let mut result = state.db.query(query)
        .bind(("termin_id", thing))
        .bind(("status", new_status))
        .bind(("approved_by", req.approver_name.clone()))
        .bind(("catatan_approval", req.catatan_approval.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let message = if req.approve {
        "Termin approved by Director. Waiting for payment by Finance.".to_string()
    } else {
        "Termin rejected by Director. Returned to draft.".to_string()
    };

    match termin {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: Some(message),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn pay_termin(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
    Json(req): Json<PayTerminRequest>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = r#"
        UPDATE $termin_id SET 
            status = 'paid',
            paid_by = $paid_by,
            paid_at = time::now(),
            jumlah_dibayar = $jumlah_dibayar,
            referensi_pembayaran = $referensi_pembayaran,
            catatan_pembayaran = $catatan_pembayaran,
            bukti_pembayaran = $bukti_pembayaran,
            updated_at = time::now()
    "#;

    let mut result = state.db.query(query)
        .bind(("termin_id", thing))
        .bind(("paid_by", req.payer_name.clone()))
        .bind(("jumlah_dibayar", req.jumlah_dibayar))
        .bind(("referensi_pembayaran", req.referensi_pembayaran.clone()))
        .bind(("catatan_pembayaran", req.catatan_pembayaran.clone()))
        .bind(("bukti_pembayaran", req.bukti_pembayaran.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match termin {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: Some("Payment confirmed. Termin completed.".to_string()),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_termin(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Cascade delete: first delete associated files
    let delete_files = "DELETE termin_files WHERE termin_id = $termin_id";
    state.db.query(delete_files)
        .bind(("termin_id", thing.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Then delete the termin
    let delete_termin = "DELETE $termin_id";
    state.db.query(delete_termin)
        .bind(("termin_id", thing))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        message: Some("Termin and associated files deleted successfully".to_string()),
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
