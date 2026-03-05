use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use calamine::{Reader, Xlsx, open_workbook_from_rs, Data};
use std::sync::Arc;
use std::io::Cursor;
use chrono::NaiveDate;
use surrealdb::sql::Thing;

use crate::models::{
    ApiResponse, Project, Site, ProjectType,
    BulkImportExcelResponse, ImportError, ImportSummary,
};
use crate::state::AppState;

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
        Data::DateTime(dt) => format!("{}", dt),
        Data::Bool(b) => b.to_string(),
        Data::Empty => String::new(),
        _ => String::new(),
    }
}

// Helper to parse cell as i64 (for budget fields)
fn get_cell_i64(row: &[Data], col_idx: usize) -> i64 {
    if col_idx >= row.len() {
        return 0;
    }
    match &row[col_idx] {
        Data::Int(i) => *i,
        Data::Float(f) => *f as i64,
        Data::String(s) => {
            // Remove any formatting (commas, dots, etc)
            let cleaned = s.replace(",", "").replace(".", "").replace(" ", "");
            cleaned.parse::<i64>().unwrap_or(0)
        }
        _ => 0,
    }
}

// Helper to parse date
fn parse_date_field(row: &[Data], col_idx: usize) -> String {
    if col_idx >= row.len() {
        return chrono::Utc::now().format("%Y-%m-%d").to_string();
    }
    
    match &row[col_idx] {
        Data::DateTime(dt) => {
            if let Some(datetime) = dt.as_datetime() {
                datetime.format("%Y-%m-%d").to_string()
            } else {
                chrono::Utc::now().format("%Y-%m-%d").to_string()
            }
        }
        Data::String(s) => {
            // Try to parse various date formats
            if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                return date.format("%Y-%m-%d").to_string();
            }
            if let Ok(date) = NaiveDate::parse_from_str(s, "%d/%m/%Y") {
                return date.format("%Y-%m-%d").to_string();
            }
            // Fallback: return as-is or current date
            if !s.is_empty() {
                s.clone()
            } else {
                chrono::Utc::now().format("%Y-%m-%d").to_string()
            }
        }
        _ => chrono::Utc::now().format("%Y-%m-%d").to_string(),
    }
}

pub async fn bulk_import_from_excel(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<BulkImportExcelResponse>>, StatusCode> {
    
    // Step 1: Extract file from multipart
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            filename = field.file_name().map(|s| s.to_string());
            let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            file_data = Some(bytes.to_vec());
        }
    }
    
    let file_bytes = file_data.ok_or(StatusCode::BAD_REQUEST)?;
    let file_name = filename.unwrap_or_else(|| "unknown.xlsx".to_string());
    
    // Step 2: Parse filename to extract project info
    // Filename format: "OSP Project Report_Update-20260215-PEKALONGAN.xlsx"
    let project_lokasi = file_name
        .replace(".xlsx", "")
        .split('-')
        .last()
        .unwrap_or("Unknown")
        .trim()
        .to_string();
    
    let project_name = format!("OSP Project {}", project_lokasi);
    
    // Extract date from filename if possible
    let date_parts: Vec<&str> = file_name.split('-').collect();
    let project_date = if date_parts.len() >= 2 {
        let date_str = date_parts[date_parts.len() - 2];
        if date_str.len() == 8 {
            // Format: YYYYMMDD
            format!("{}-{}-{}", &date_str[0..4], &date_str[4..6], &date_str[6..8])
        } else {
            chrono::Utc::now().format("%Y-%m-%d").to_string()
        }
    } else {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    };
    
    // Step 3: Parse Excel file
    let cursor = Cursor::new(file_bytes);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Get sheet "Active Project Details" (index 2, 0-based)
    let sheet_names = workbook.sheet_names().to_vec();
    let target_sheet = if sheet_names.len() > 2 {
        &sheet_names[2]
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };
    
    let range = workbook
        .worksheet_range(target_sheet)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Step 4: Get project budget from Row 2
    let rows: Vec<_> = range.rows().collect();
    
    let total_boq_aktual = if rows.len() > 1 {
        get_cell_i64(rows[1], 8) // Column I2 (index 8)
    } else {
        0
    };
    
    let total_nilai_po = if rows.len() > 1 {
        get_cell_i64(rows[1], 12) // Column M2 (index 12)
    } else {
        0
    };
    
    // Step 5: Create Project
    let new_project = Project {
        id: None,
        name: project_name.clone(),
        lokasi: project_lokasi.clone(),
        value: total_boq_aktual,
        cost: total_nilai_po,
        tipe: ProjectType::BebanOperasional,
        status: Some("active".to_string()),
        tgi_start: Some(project_date.clone()),
        tgi_end: None,
        keterangan: "Progress Projek OSP Telkom Akses - Import from Excel".to_string(),
        created_at: None,
        updated_at: None,
    };
    
    let project: Option<Project> = state
        .db
        .create("projects")
        .content(new_project)
        .await
        .map_err(|e| {
            eprintln!("Database error creating project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let project = project.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let project_id_str = project.id.as_ref()
        .map(|t| t.to_string())
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Prepare Thing for project_id
    let project_id_clean = strip_table_prefix(&project_id_str, "projects");
    let project_thing = Thing::try_from(("projects", project_id_clean))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Step 6: Parse Sites from Row 6 onwards (after header row 5)
    let mut created_sites: Vec<Site> = Vec::new();
    let mut errors: Vec<ImportError> = Vec::new();
    
    // Header is in row 5 (index 4), data starts at row 6 (index 5)
    for (idx, row) in rows.iter().enumerate().skip(5) {
        let row_number = idx + 1;
        
        // Check if row has data (check if NO column has value)
        let no_value = get_cell_string(row, 0); // Column A
        if no_value.is_empty() {
            continue; // Skip empty rows
        }
        
        // Extract fields from columns
        // L: NAMA LOP [SITE] - site_name
        let site_name = get_cell_string(row, 11);
        if site_name.is_empty() {
            errors.push(ImportError {
                row_number,
                field: "site_name".to_string(),
                message: "Site name (Column L) is required but empty".to_string(),
                data: None,
            });
            continue;
        }
        
        // D: WTIEL - lokasi
        let lokasi = get_cell_string(row, 3);
        
        // K: NAMA PO - pekerjaan
        let pekerjaan = get_cell_string(row, 10);
        
        // J: NOMOR PO - nomor_kontrak
        let nomor_kontrak = get_cell_string(row, 9);
        let nomor_kontrak = if nomor_kontrak.is_empty() {
            format!("PO-{}-{}", row_number, chrono::Utc::now().timestamp())
        } else {
            nomor_kontrak
        };
        
        // G: TANGGAL WO - start
        let start = parse_date_field(row, 6);
        
        // O: TANGGAL - end (or use start if empty)
        let end_date = parse_date_field(row, 14);
        let end = if end_date.len() > 8 { end_date } else { start.clone() };
        
        // M: NILAI PO - maximal_budget
        let maximal_budget = get_cell_i64(row, 12);
        
        // H: BOQ KONTRAK - cost_estimated
        let cost_estimated = get_cell_i64(row, 7);
        
        // B: TIPE PROJECT, N: LAST STATUS, P: KETERANGAN - combine for site_info
        let tipe_project = get_cell_string(row, 1);
        let last_status = get_cell_string(row, 13);
        let keterangan = get_cell_string(row, 15);
        let site_info = format!("{} | Status: {} | {}", tipe_project, last_status, keterangan);
        
        // Defaults
        let pemberi_tugas = "PT Telkom Indonesia".to_string();
        let penerima_tugas = "Vendor/Pelaksana".to_string();
        
        // Create Site
        let create_site_query = "CREATE sites SET 
            project_id = type::thing($project_id),
            site_name = $site_name,
            site_info = $site_info,
            pekerjaan = $pekerjaan,
            lokasi = $lokasi,
            latitude = $latitude,
            longitude = $longitude,
            nomor_kontrak = $nomor_kontrak,
            start = $start,
            end = $end,
            maximal_budget = $maximal_budget,
            cost_estimated = $cost_estimated,
            pemberi_tugas = $pemberi_tugas,
            penerima_tugas = $penerima_tugas,
            site_document = $site_document,
            created_at = time::now(),
            updated_at = time::now()";
        
        match state.db.query(create_site_query)
            .bind(("project_id", project_thing.clone()))
            .bind(("site_name", site_name.clone()))
            .bind(("site_info", site_info))
            .bind(("pekerjaan", pekerjaan))
            .bind(("lokasi", lokasi))
            .bind(("latitude", None::<String>))
            .bind(("longitude", None::<String>))
            .bind(("nomor_kontrak", nomor_kontrak))
            .bind(("start", start))
            .bind(("end", end))
            .bind(("maximal_budget", maximal_budget))
            .bind(("cost_estimated", cost_estimated))
            .bind(("pemberi_tugas", pemberi_tugas))
            .bind(("penerima_tugas", penerima_tugas))
            .bind(("site_document", None::<String>))
            .await
        {
            Ok(mut response) => {
                match response.take::<Vec<Site>>(0) {
                    Ok(sites) => {
                        if let Some(site) = sites.into_iter().next() {
                            created_sites.push(site);
                        }
                    }
                    Err(e) => {
                        errors.push(ImportError {
                            row_number,
                            field: "database".to_string(),
                            message: format!("Failed to parse site: {}", e),
                            data: Some(serde_json::json!({"site_name": site_name})),
                        });
                    }
                }
            }
            Err(e) => {
                errors.push(ImportError {
                    row_number,
                    field: "database".to_string(),
                    message: format!("Failed to create site: {}", e),
                    data: Some(serde_json::json!({"site_name": site_name})),
                });
            }
        }
    }
    
    // Step 7: Build response
    let total_rows = rows.len() - 5; // Subtract header rows
    let summary = ImportSummary {
        project_id: project_id_str.clone(),
        project_name: project_name.clone(),
        total_budget: total_boq_aktual,
        sites_count: created_sites.len(),
        message: format!(
            "Import completed: {} sites created, {} failed out of {} rows",
            created_sites.len(),
            errors.len(),
            total_rows
        ),
    };
    
    let response = BulkImportExcelResponse {
        project,
        total_rows,
        sites_created: created_sites.len(),
        sites_failed: errors.len(),
        created_sites,
        errors,
        summary,
    };
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: Some("Bulk import from Excel completed successfully".to_string()),
    }))
}
