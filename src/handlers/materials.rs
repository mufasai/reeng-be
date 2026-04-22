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
            material_master_id: $material_master_id,
            source_master: $source_master,
            harga_satuan: $harga_satuan,
            spesifikasi: $spesifikasi,
            satuan: $satuan,
            material_type: $material_type,
            direction: $direction,
            delivery_note_no: $delivery_note_no,
            po_delivery_date: $po_delivery_date,
            vendor: $vendor,
            sender: $sender,
            receiver: $receiver,
            keterangan: $keterangan,
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
        .bind(("material_master_id", req.material_master_id.clone()))
        .bind(("source_master", req.source_master.unwrap_or(false)))
        .bind(("harga_satuan", req.harga_satuan))
        .bind(("spesifikasi", req.spesifikasi.clone()))
        .bind(("satuan", req.satuan.clone()))
        .bind(("material_type", req.material_type.clone()))
        .bind(("direction", req.direction.clone()))
        .bind(("delivery_note_no", req.delivery_note_no.clone()))
        .bind(("po_delivery_date", req.po_delivery_date.clone()))
        .bind(("vendor", req.vendor.clone()))
        .bind(("sender", req.sender.clone()))
        .bind(("receiver", req.receiver.clone()))
        .bind(("keterangan", req.keterangan.clone()))
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
                material_master_id: $material_master_id,
                source_master: $source_master,
                harga_satuan: $harga_satuan,
                spesifikasi: $spesifikasi,
                satuan: $satuan,
                material_type: $material_type,
                direction: $direction,
                delivery_note_no: $delivery_note_no,
                po_delivery_date: $po_delivery_date,
                vendor: $vendor,
                sender: $sender,
                receiver: $receiver,
                keterangan: $keterangan,
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
            .bind(("material_master_id", item.material_master_id.clone()))
            .bind(("source_master", item.source_master.unwrap_or(false)))
            .bind(("harga_satuan", item.harga_satuan))
            .bind(("spesifikasi", item.spesifikasi.clone()))
            .bind(("satuan", item.satuan.clone()))
            .bind(("material_type", item.material_type.clone()))
            .bind(("direction", item.direction.clone()))
            .bind(("delivery_note_no", item.delivery_note_no.clone()))
            .bind(("po_delivery_date", item.po_delivery_date.clone()))
            .bind(("vendor", item.vendor.clone()))
            .bind(("sender", item.sender.clone()))
            .bind(("receiver", item.receiver.clone()))
            .bind(("keterangan", item.keterangan.clone()))
            .await
            .map_err(|e| {
                eprintln!("DB error bulk_create_materials: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

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
