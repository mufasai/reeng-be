use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::models::{ApiResponse, Cost, CreateCostRequest};
use crate::state::AppState;

pub async fn create_cost(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCostRequest>,
) -> Result<Json<ApiResponse<Cost>>, StatusCode> {
    let query = r#"
        CREATE costs CONTENT {
            project_id: type::thing($project_id),
            site_id: type::thing($site_id),
            type_termin: $type_termin,
            tgl_pengajuan: $tgl_pengajuan,
            jumlah_pengajuan: $jumlah_pengajuan,
            status: $status,
            catatan_tolak: $catatan_tolak,
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let status = req.status.unwrap_or_else(|| "pending".to_string());

    let mut result = state.db.query(query)
        .bind(("project_id", req.project_id.clone()))
        .bind(("site_id", req.site_id.clone()))
        .bind(("type_termin", req.type_termin.clone()))
        .bind(("tgl_pengajuan", req.tgl_pengajuan.clone()))
        .bind(("jumlah_pengajuan", req.jumlah_pengajuan))
        .bind(("status", status))
        .bind(("catatan_tolak", req.catatan_tolak.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cost: Option<Cost> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match cost {
        Some(cost) => Ok(Json(ApiResponse {
            success: true,
            data: Some(cost),
            message: Some("Cost created successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_costs(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Cost>>>, StatusCode> {
    let query = "SELECT * FROM costs ORDER BY created_at DESC";

    let mut result = state.db.query(query)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let costs: Vec<Cost> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(costs),
        message: None,
    }))
}

pub async fn get_costs_by_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Cost>>>, StatusCode> {
    let query = "SELECT * FROM costs WHERE project_id = type::thing('projects', $id) ORDER BY created_at DESC";

    let mut result = state.db.query(query)
        .bind(("id", project_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let costs: Vec<Cost> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(costs),
        message: None,
    }))
}

pub async fn get_costs_by_site(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Cost>>>, StatusCode> {
    let query = "SELECT * FROM costs WHERE site_id = type::thing('sites', $id) ORDER BY created_at DESC";

    let mut result = state.db.query(query)
        .bind(("id", site_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let costs: Vec<Cost> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(costs),
        message: None,
    }))
}

pub async fn approve_cost(
    State(state): State<Arc<AppState>>,
    Path(cost_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<Cost>>, StatusCode> {
    let acc_by = body.get("acc_by").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let acc_name = body.get("acc_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let jumlah_acc = body.get("jumlah_acc").and_then(|v| v.as_i64()).unwrap_or(0);

    let query = r#"
        UPDATE type::thing('costs', $id) SET
            status = 'approved',
            acc_by = $acc_by,
            acc_name = $acc_name,
            jumlah_acc = $jumlah_acc,
            tgl_acc = time::now(),
            updated_at = time::now()
    "#;

    let mut result = state.db.query(query)
        .bind(("id", cost_id))
        .bind(("acc_by", acc_by))
        .bind(("acc_name", acc_name))
        .bind(("jumlah_acc", jumlah_acc))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cost: Option<Cost> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match cost {
        Some(cost) => Ok(Json(ApiResponse {
            success: true,
            data: Some(cost),
            message: Some("Cost approved successfully".to_string()),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}
