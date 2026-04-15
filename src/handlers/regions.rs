use crate::extractors::FormOrJson;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::models::{ApiResponse, Area, Region, CreateAreaRequest, CreateRegionRequest};
use crate::state::AppState;

// ==================== AREA HANDLERS ====================

pub async fn create_area(
    State(state): State<Arc<AppState>>,
    FormOrJson(req): FormOrJson<CreateAreaRequest>,
) -> Result<Json<ApiResponse<Area>>, StatusCode> {
    let query = r#"
        CREATE areas CONTENT {
            nama_area: $nama_area,
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let mut result = state.db.query(query)
        .bind(("nama_area", req.nama_area.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let area: Option<Area> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match area {
        Some(area) => Ok(Json(ApiResponse {
            success: true,
            data: Some(area),
            message: Some("Area created successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_areas(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Area>>>, StatusCode> {
    let query = "SELECT * FROM areas ORDER BY nama_area ASC";

    let mut result = state.db.query(query)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let areas: Vec<Area> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(areas),
        message: None,
    }))
}

// ==================== REGION HANDLERS ====================

pub async fn create_region(
    State(state): State<Arc<AppState>>,
    FormOrJson(req): FormOrJson<CreateRegionRequest>,
) -> Result<Json<ApiResponse<Region>>, StatusCode> {
    let query = r#"
        CREATE regions CONTENT {
            area_id: type::thing($area_id),
            kode_region: $kode_region,
            nama_region: $nama_region,
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let mut result = state.db.query(query)
        .bind(("area_id", req.area_id.clone()))
        .bind(("kode_region", req.kode_region.clone()))
        .bind(("nama_region", req.nama_region.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let region: Option<Region> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match region {
        Some(region) => Ok(Json(ApiResponse {
            success: true,
            data: Some(region),
            message: Some("Region created successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_regions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Region>>>, StatusCode> {
    let query = "SELECT * FROM regions ORDER BY kode_region ASC";

    let mut result = state.db.query(query)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let regions: Vec<Region> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(regions),
        message: None,
    }))
}

pub async fn get_regions_by_area(
    State(state): State<Arc<AppState>>,
    Path(area_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Region>>>, StatusCode> {
    let query = "SELECT * FROM regions WHERE area_id = type::thing('areas', $id) ORDER BY kode_region ASC";

    let mut result = state.db.query(query)
        .bind(("id", area_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let regions: Vec<Region> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(regions),
        message: None,
    }))
}
