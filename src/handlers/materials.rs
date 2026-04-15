use crate::extractors::FormOrJson;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::models::{ApiResponse, Material, CreateMaterialRequest};
use crate::state::AppState;

pub async fn create_material(
    State(state): State<Arc<AppState>>,
    FormOrJson(req): FormOrJson<CreateMaterialRequest>,
) -> Result<Json<ApiResponse<Material>>, StatusCode> {
    let query = r#"
        CREATE materials CONTENT {
            skp: $skp,
            name: $name,
            unit: $unit,
            qty: $qty,
            project_id: type::thing($project_id),
            site_id: type::thing($site_id),
            tgl: $tgl,
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let mut result = state.db.query(query)
        .bind(("skp", req.skp.clone()))
        .bind(("name", req.name.clone()))
        .bind(("unit", req.unit.clone()))
        .bind(("qty", req.qty))
        .bind(("project_id", req.project_id.clone()))
        .bind(("site_id", req.site_id.clone()))
        .bind(("tgl", req.tgl.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let material: Option<Material> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match material {
        Some(material) => Ok(Json(ApiResponse {
            success: true,
            data: Some(material),
            message: Some("Material created successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_materials(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Material>>>, StatusCode> {
    let query = "SELECT * FROM materials ORDER BY created_at DESC";

    let mut result = state.db.query(query)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let materials: Vec<Material> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(materials),
        message: None,
    }))
}

pub async fn get_materials_by_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Material>>>, StatusCode> {
    let query = "SELECT * FROM materials WHERE project_id = type::thing('projects', $id) ORDER BY created_at DESC";

    let mut result = state.db.query(query)
        .bind(("id", project_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let materials: Vec<Material> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(materials),
        message: None,
    }))
}

pub async fn get_materials_by_site(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Material>>>, StatusCode> {
    let query = "SELECT * FROM materials WHERE site_id = type::thing('sites', $id) ORDER BY created_at DESC";

    let mut result = state.db.query(query)
        .bind(("id", site_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let materials: Vec<Material> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(materials),
        message: None,
    }))
}

pub async fn bulk_create_materials(
    State(state): State<Arc<AppState>>,
    FormOrJson(req): FormOrJson<crate::models::BulkCreateMaterialRequest>,
) -> Result<Json<ApiResponse<Vec<Material>>>, StatusCode> {
    let mut created_materials = Vec::new();
    
    for item in req.materials {
        let query = r#"
            CREATE materials CONTENT {
                skp: $skp,
                name: $name,
                unit: $unit,
                qty: $qty,
                project_id: type::thing($project_id),
                site_id: type::thing($site_id),
                tgl: $tgl,
                created_at: time::now(),
                updated_at: time::now()
            }
        "#;

        let mut result = state.db.query(query)
            .bind(("skp", item.skp.clone()))
            .bind(("name", item.name.clone()))
            .bind(("unit", item.unit.clone()))
            .bind(("qty", item.qty))
            .bind(("project_id", req.project_id.clone()))
            .bind(("site_id", req.site_id.clone()))
            .bind(("tgl", item.tgl.clone()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Ok(Some(material)) = result.take::<Option<Material>>(0) {
            created_materials.push(material);
        }
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(created_materials),
        message: Some("Bulk materials created successfully".to_string()),
    }))
}
