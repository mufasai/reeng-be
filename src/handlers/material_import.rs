use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use calamine::{Reader, Xlsx, open_workbook_from_rs, Data};
use std::sync::Arc;
use std::io::Cursor;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use surrealdb::sql::Thing;

use crate::models::{
    ApiResponse, MaterialMaster, Project, ProjectType,
    MaterialImportResponse, ImportError,
};
use crate::state::AppState;

// Helper to calculate file hash for duplicate detection
// Helper function to strip table prefix
fn strip_table_prefix<'a>(id_str: &'a str, table: &str) -> &'a str {
    let prefix = format!("{}:", table);
    id_str.strip_prefix(&prefix).unwrap_or(id_str)
}

// Helper to get cell value as string
fn get_cell_string(row: &[Data], col_idx: usize) -> String {
    if col_idx >= row.len() {
        return String::new();
    }
    match &row[col_idx] {
        Data::String(s) => s.trim().to_string(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => f.to_string(),
        Data::DateTime(dt) => {
            if let Some(datetime) = dt.as_datetime() {
                datetime.format("%Y-%m-%d").to_string()
            } else {
                dt.to_string()
            }
        },
        Data::Bool(b) => b.to_string(),
        Data::Empty => String::new(),
        _ => String::new(),
    }
}

// Helper to parse cell as i64
fn get_cell_i64(row: &[Data], col_idx: usize) -> i64 {
    if col_idx >= row.len() {
        return 0;
    }
    match &row[col_idx] {
        Data::Int(i) => *i,
        Data::Float(f) => *f as i64,
        Data::String(s) => {
            let cleaned = s.replace(",", "").replace(".", "").replace(" ", "");
            cleaned.parse::<i64>().unwrap_or(0)
        }
        _ => 0,
    }
}

pub async fn import_materials_from_excel(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<MaterialImportResponse>>, StatusCode> {
    
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut project_id_override: Option<String> = None;
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            filename = field.file_name().map(|s| s.to_string());
            let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            file_data = Some(bytes.to_vec());
        } else if name == "project_id" {
            let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            if !value.trim().is_empty() {
                project_id_override = Some(value);
            }
        }
    }
    
    let file_bytes = file_data.ok_or(StatusCode::BAD_REQUEST)?;
    let _file_name = filename.unwrap_or_else(|| "unknown.xlsx".to_string());
    
    // Parse Excel
    let cursor = Cursor::new(file_bytes);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let sheet_names = workbook.sheet_names().to_vec();
    
    // Look for Sheet 4 "INVENTORY REPORT" or similar
    let sheet_name = sheet_names.iter().find(|s| s.to_uppercase().contains("INVENTORY")).cloned()
        .or_else(|| if sheet_names.len() >= 4 { Some(sheet_names[3].clone()) } else { None })
        .ok_or(StatusCode::BAD_REQUEST)?;
        
    let range = workbook.worksheet_range(&sheet_name)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let rows: Vec<_> = range.rows().collect();
    if rows.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Identify or Create Project
    let (project_id_str, project_name) = if let Some(pid) = project_id_override {
        // Fetch project name
        let p: Option<Project> = state.db.select(("projects", strip_table_prefix(&pid, "projects"))).await.unwrap_or(None);
        (pid, p.map(|x| x.name).unwrap_or("Existing Project".to_string()))
    } else {
        // Create a default project for this import
        let p_name = format!("Material Import {}", chrono::Utc::now().format("%Y-%m-%d"));
        let new_p = Project {
            id: None,
            name: p_name.clone(),
            lokasi: "Imported".to_string(),
            value: 0,
            cost: 0,
            tipe: ProjectType::BebanOperasional,
            status: Some("active".to_string()),
            tgi_start: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
            tgi_end: None,
            keterangan: "Imported from Material Excel".to_string(),
            regional: None,
            created_at: None,
            updated_at: None,
        };
        let created_p: Option<Project> = state.db.create("projects").content(new_p).await.unwrap_or(None);
        let p = created_p.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        (p.id.unwrap().to_string(), p.name)
    };
    
    let project_id_clean = strip_table_prefix(&project_id_str, "projects");
    let project_thing = Thing::try_from(("projects", project_id_clean))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut materials_created = 0;
    let mut masters_created = 0;
    let mut errors = Vec::new();
    
    // Skip header row (index 0)
    for (idx, row) in rows.iter().enumerate().skip(1) {
        let row_number = idx + 1;
        
        let material_name = get_cell_string(row, 4);
        if material_name.is_empty() {
            continue; // Skip empty rows
        }
        
        let material_type = get_cell_string(row, 1);
        let direction = get_cell_string(row, 2);
        let qty = get_cell_i64(row, 3);
        let tgl = get_cell_string(row, 5);
        let delivery_note_no = get_cell_string(row, 6);
        let po_delivery_date = get_cell_string(row, 7);
        let vendor = get_cell_string(row, 8);
        let sender = get_cell_string(row, 9);
        let receiver = get_cell_string(row, 10);
        
        // 1. Manage MaterialMaster
        let mut master_id = None;
        let master_query = "SELECT * FROM material_master WHERE nama_material = $name LIMIT 1";
        if let Ok(mut res) = state.db.query(master_query).bind(("name", material_name.clone())).await {
            let masters: Vec<MaterialMaster> = res.take(0).unwrap_or_default();
            if let Some(m) = masters.into_iter().next() {
                master_id = m.id.map(|t| t.to_string());
            } else {
                // Create new master
                let new_master = MaterialMaster {
                    id: None,
                    kode_material: None,
                    nama_material: material_name.clone(),
                    kategori: Some(material_type.clone()),
                    spesifikasi: None,
                    satuan: Some("pcs".to_string()),
                    harga_satuan: Some(0),
                    keterangan: Some("Imported from Excel".to_string()),
                    status_aktif: true,
                    created_by: Some("system".to_string()),
                    created_at: None,
                    updated_at: None,
                };
                if let Ok(Some(m)) = state.db.create::<Option<MaterialMaster>>("material_master").content(new_master).await {
                    master_id = m.id.map(|t| t.to_string());
                    masters_created += 1;
                }
            }
        }
        
        // 2. Create Material entry
        let query = r#"
            CREATE materials CONTENT {
                name: $name,
                material_type: $material_type,
                direction: $direction,
                qty: $qty,
                tgl: $tgl,
                delivery_note_no: $delivery_note_no,
                po_delivery_date: $po_delivery_date,
                vendor: $vendor,
                sender: $sender,
                receiver: $receiver,
                project_id: $project_id,
                material_master_id: $master_id,
                unit: "pcs",
                created_at: time::now(),
                updated_at: time::now()
            }
        "#;
        
        match state.db.query(query)
            .bind(("name", material_name.clone()))
            .bind(("material_type", material_type))
            .bind(("direction", direction))
            .bind(("qty", qty))
            .bind(("tgl", tgl))
            .bind(("delivery_note_no", delivery_note_no))
            .bind(("po_delivery_date", po_delivery_date))
            .bind(("vendor", vendor))
            .bind(("sender", sender))
            .bind(("receiver", receiver))
            .bind(("project_id", project_thing.clone()))
            .bind(("master_id", master_id))
            .await
        {
            Ok(_) => materials_created += 1,
            Err(e) => {
                errors.push(ImportError {
                    row_number,
                    field: "database".to_string(),
                    message: format!("Failed to create material: {}", e),
                    data: Some(serde_json::json!({"name": material_name})),
                });
            }
        }
    }
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(MaterialImportResponse {
            project_id: project_id_str,
            project_name,
            total_rows: rows.len() - 1,
            materials_created,
            masters_created,
            errors,
        }),
        message: Some(format!("Imported {} materials for project", materials_created)),
    }))
}
