use crate::extractors::FormOrJson;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::models::{ApiResponse, MaterialMaster, CreateMaterialMasterRequest, UpdateMaterialMasterRequest};
use crate::state::AppState;

// ==================== CREATE ====================

pub async fn create_material_master(
    State(state): State<Arc<AppState>>,
    FormOrJson(req): FormOrJson<CreateMaterialMasterRequest>,
) -> Result<Json<ApiResponse<MaterialMaster>>, StatusCode> {
    let query = r#"
        CREATE material_master CONTENT {
            kode_material:  $kode_material,
            nama_material:  $nama_material,
            kategori:       $kategori,
            spesifikasi:    $spesifikasi,
            satuan:         $satuan,
            harga_satuan:   $harga_satuan,
            keterangan:     $keterangan,
            status_aktif:   $status_aktif,
            created_by:     $created_by,
            created_at:     time::now(),
            updated_at:     time::now()
        }
    "#;

    let mut result = state.db.query(query)
        .bind(("kode_material",  req.kode_material.clone()))
        .bind(("nama_material",  req.nama_material.clone()))
        .bind(("kategori",       req.kategori.clone()))
        .bind(("spesifikasi",    req.spesifikasi.clone()))
        .bind(("satuan",         req.satuan.clone()))
        .bind(("harga_satuan",   req.harga_satuan))
        .bind(("keterangan",     req.keterangan.clone()))
        .bind(("status_aktif",   req.status_aktif.unwrap_or(true)))
        .bind(("created_by",     req.created_by.clone()))
        .await
        .map_err(|e| {
            eprintln!("DB error create_material_master: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let record: Option<MaterialMaster> = result.take(0)
        .map_err(|e| {
            eprintln!("Parse error create_material_master: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match record {
        Some(m) => Ok(Json(ApiResponse {
            success: true,
            data: Some(m),
            message: Some("Material master created successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ==================== LIST ALL ====================

pub async fn list_material_masters(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<MaterialMaster>>>, StatusCode> {
    let query = "SELECT * FROM material_master ORDER BY nama_material ASC";

    let mut result = state.db.query(query)
        .await
        .map_err(|e| {
            eprintln!("DB error list_material_masters: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let records: Vec<MaterialMaster> = result.take(0)
        .map_err(|e| {
            eprintln!("Parse error list_material_masters: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(records),
        message: None,
    }))
}

// ==================== LIST ACTIVE ONLY ====================

pub async fn list_active_material_masters(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<MaterialMaster>>>, StatusCode> {
    let query = "SELECT * FROM material_master WHERE status_aktif = true ORDER BY nama_material ASC";

    let mut result = state.db.query(query)
        .await
        .map_err(|e| {
            eprintln!("DB error list_active_material_masters: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let records: Vec<MaterialMaster> = result.take(0)
        .map_err(|e| {
            eprintln!("Parse error list_active_material_masters: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(records),
        message: None,
    }))
}

// ==================== LIST BY KATEGORI ====================

pub async fn list_material_masters_by_kategori(
    State(state): State<Arc<AppState>>,
    Path(kategori): Path<String>,
) -> Result<Json<ApiResponse<Vec<MaterialMaster>>>, StatusCode> {
    let query = "SELECT * FROM material_master WHERE kategori = $kategori AND status_aktif = true ORDER BY nama_material ASC";

    let mut result = state.db.query(query)
        .bind(("kategori", kategori))
        .await
        .map_err(|e| {
            eprintln!("DB error list_material_masters_by_kategori: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let records: Vec<MaterialMaster> = result.take(0)
        .map_err(|e| {
            eprintln!("Parse error list_material_masters_by_kategori: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(records),
        message: None,
    }))
}

// ==================== GET BY ID ====================

pub async fn get_material_master(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<MaterialMaster>>, StatusCode> {
    let query = "SELECT * FROM type::thing('material_master', $id)";

    let mut result = state.db.query(query)
        .bind(("id", id.clone()))
        .await
        .map_err(|e| {
            eprintln!("DB error get_material_master {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let records: Vec<MaterialMaster> = result.take(0)
        .map_err(|e| {
            eprintln!("Parse error get_material_master: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match records.into_iter().next() {
        Some(m) => Ok(Json(ApiResponse {
            success: true,
            data: Some(m),
            message: None,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ==================== UPDATE ====================

pub async fn update_material_master(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    FormOrJson(req): FormOrJson<UpdateMaterialMasterRequest>,
) -> Result<Json<ApiResponse<MaterialMaster>>, StatusCode> {
    // Fetch existing record first
    let fetch_query = "SELECT * FROM type::thing('material_master', $id)";
    let mut fetch_res = state.db.query(fetch_query)
        .bind(("id", id.clone()))
        .await
        .map_err(|e| {
            eprintln!("DB error fetch material_master {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let records: Vec<MaterialMaster> = fetch_res.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if records.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut existing = records.into_iter().next().unwrap();

    // Apply partial updates
    if let Some(v) = req.kode_material  { existing.kode_material  = Some(v); }
    if let Some(v) = req.nama_material  { existing.nama_material   = v; }
    if let Some(v) = req.kategori       { existing.kategori        = Some(v); }
    if let Some(v) = req.spesifikasi    { existing.spesifikasi     = Some(v); }
    if let Some(v) = req.satuan         { existing.satuan          = Some(v); }
    if let Some(v) = req.harga_satuan   { existing.harga_satuan    = Some(v); }
    if let Some(v) = req.keterangan     { existing.keterangan      = Some(v); }
    if let Some(v) = req.status_aktif   { existing.status_aktif    = v; }

    let update_query = r#"
        UPDATE type::thing('material_master', $id) SET
            kode_material = $kode_material,
            nama_material = $nama_material,
            kategori      = $kategori,
            spesifikasi   = $spesifikasi,
            satuan        = $satuan,
            harga_satuan  = $harga_satuan,
            keterangan    = $keterangan,
            status_aktif  = $status_aktif,
            updated_at    = time::now()
    "#;

    let mut update_res = state.db.query(update_query)
        .bind(("id",            id.clone()))
        .bind(("kode_material", existing.kode_material.clone()))
        .bind(("nama_material", existing.nama_material.clone()))
        .bind(("kategori",      existing.kategori.clone()))
        .bind(("spesifikasi",   existing.spesifikasi.clone()))
        .bind(("satuan",        existing.satuan.clone()))
        .bind(("harga_satuan",  existing.harga_satuan))
        .bind(("keterangan",    existing.keterangan.clone()))
        .bind(("status_aktif",  existing.status_aktif))
        .await
        .map_err(|e| {
            eprintln!("DB error update_material_master {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let updated: Vec<MaterialMaster> = update_res.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(updated.into_iter().next().unwrap_or(existing)),
        message: Some("Material master updated successfully".to_string()),
    }))
}

// ==================== SOFT DELETE (deactivate) ====================

pub async fn deactivate_material_master(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let query = "UPDATE type::thing('material_master', $id) SET status_aktif = false, updated_at = time::now()";

    state.db.query(query)
        .bind(("id", id.clone()))
        .await
        .map_err(|e| {
            eprintln!("DB error deactivate_material_master {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(format!("Material master {} deactivated", id)),
        message: Some("Material master deactivated (status_aktif = false)".to_string()),
    }))
}

// ==================== HARD DELETE ====================

pub async fn delete_material_master(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let query = "DELETE type::thing('material_master', $id)";

    state.db.query(query)
        .bind(("id", id.clone()))
        .await
        .map_err(|e| {
            eprintln!("DB error delete_material_master {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(format!("Material master {} deleted", id)),
        message: Some("Material master deleted successfully".to_string()),
    }))
}
