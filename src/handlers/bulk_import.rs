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

// Helper to parse ProjectType from string
fn parse_project_type(type_str: &str) -> ProjectType {
    let normalized = type_str.to_uppercase().trim().to_string();
    
    match normalized.as_str() {
        "COMBAT" => ProjectType::Combat,
        "L2H" => ProjectType::L2h,
        "BLACK SITE" | "BLACKSITE" => ProjectType::BlackSite,
        "REFINEN" => ProjectType::Refinen,
        "FILTER" => ProjectType::Filter,
        "BEBAN OPERASIONAL" | "BEBANOPERASIONAL" => ProjectType::BebanOperasional,
        _ => ProjectType::BebanOperasional, // Default fallback
    }
}

pub async fn bulk_import_from_excel(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<BulkImportExcelResponse>>, StatusCode> {
    
    // Step 1: Extract file from multipart
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;
    let mut project_type_override: Option<String> = None;
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            filename = field.file_name().map(|s| s.to_string());
            let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            file_data = Some(bytes.to_vec());
        } else if name == "project_type" {
            let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            if !value.trim().is_empty() {
                project_type_override = Some(value);
            }
        }
    }
    
    let file_bytes = file_data.ok_or(StatusCode::BAD_REQUEST)?;
    let file_name = filename.unwrap_or_else(|| "unknown.xlsx".to_string());
    
    // Step 2: Parse filename to extract project info
    let file_name_clean = file_name.replace(".xlsx", "").replace(".XLSX", "");
    let parts: Vec<&str> = file_name_clean.split('_').collect();
    
    let project_lokasi = if parts.len() > 0 {
        parts[parts.len() - 1].trim().to_string()
    } else {
        let dash_parts: Vec<&str> = file_name_clean.split('-').collect();
        dash_parts[dash_parts.len() - 1].trim().to_string()
    };
    
    // Extract date from filename (YYYYMMDD format)
    let project_date = if let Some(date_part) = file_name.split(|c| c == '_' || c == '-')
        .find(|part| part.len() == 8 && part.chars().all(|c| c.is_numeric())) {
        format!("{}-{}-{}", &date_part[0..4], &date_part[4..6], &date_part[6..8])
    } else {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    };
    
    // Extract project type hint from filename (FILTER, COMBAT, etc.)
    let filename_upper = file_name.to_uppercase();
    let project_type_hint = if let Some(ref over) = project_type_override {
        over.as_str()
    } else if filename_upper.contains("FILTER") {
        "FILTER"
    } else if filename_upper.contains("COMBAT") {
        "COMBAT"
    } else if filename_upper.contains("L2H") {
        "L2H"
    } else if filename_upper.contains("BLACKSITE") || filename_upper.contains("BLACK_SITE") {
        "BLACK SITE"
    } else if filename_upper.contains("REFINEN") {
        "REFINEN"
    } else {
        "BEBAN OPERASIONAL"
    };
    
    // Step 3: Parse Excel file
    let cursor = Cursor::new(file_bytes);
    let workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let sheet_names = workbook.sheet_names().to_vec();
    
    // Step 4: Detect Excel format
    let is_eproc_format = sheet_names.len() == 1 && sheet_names[0] == "Sheet1";
    
    if is_eproc_format {
        // EPROC Format: Single sheet, Row 2 = headers, Row 3+ = data
        parse_eproc_format(state, workbook, file_name, project_lokasi, project_date, project_type_hint).await
    } else if sheet_names.len() >= 3 {
        // Old Format: Multi-sheet, Sheet 3 "Active Project Details"
        parse_old_format(state, workbook, sheet_names, file_name, project_lokasi, project_date, project_type_hint).await
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }
}

// Parse EPROC format (1 sheet, Row 2=headers, Row 3+=data)
async fn parse_eproc_format(
    state: Arc<AppState>,
    mut workbook: Xlsx<Cursor<Vec<u8>>>,
    file_name: String,
    project_lokasi: String,
    project_date: String,
    project_type_hint: &str,
) -> Result<Json<ApiResponse<BulkImportExcelResponse>>, StatusCode> {
    
    let sheet_names = workbook.sheet_names().to_vec();
    let range = workbook
        .worksheet_range(&sheet_names[0])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let rows: Vec<_> = range.rows().collect();
    
    if rows.len() < 3 {
        return Err(StatusCode::BAD_REQUEST); // Need at least headers + 1 data row
    }
    
    // Calamine skips empty rows, so Row 0 = headers, Row 1+ = data
    // Determine column mappings by inspecting headers (Row 0, index 0)
    let headers = &rows[0];
    
    // Find column indices
    let mut col_site_id = None;
    let mut col_site_name = None;
    let mut col_region = None;
    let mut col_pekerjaan = None;
    let mut col_total_price = None;
    let mut col_nop = None;
    let mut col_mitra = None;
    
    for (idx, cell) in headers.iter().enumerate() {
        let header = match cell {
            Data::String(s) => s.to_uppercase(),
            _ => continue,
        };
        
        match header.as_str() {
            "SITE_ID" => col_site_id = Some(idx),
            "SITE_NAME" => col_site_name = Some(idx),
            "REGION" => col_region = Some(idx),
            "SOW PEKERJAAN" | "SOW_EQP" | "SOW EQP" => col_pekerjaan = Some(idx),
            "TOTAL PRICE" => col_total_price = Some(idx),
            "NOP" => col_nop = Some(idx),
            "MITRA" | "MITR" => col_mitra = Some(idx),
            _ => {}
        }
    }
    
    
    // Use filename hint for project type
    let project_type = parse_project_type(project_type_hint);
    let project_name = format!("{} Project {}", project_type_hint, project_lokasi);
    
    // Calculate total budget from all sites
    let total_budget: i64 = if let Some(price_col) = col_total_price {
        rows.iter().skip(1).map(|row| get_cell_i64(row, price_col)).sum()
    } else {
        0
    };
    
    // Create Project
    let new_project = Project {
        id: None,
        name: project_name.clone(),
        lokasi: project_lokasi.clone(),
        value: total_budget,
        cost: total_budget,
        tipe: project_type.clone(),
        status: Some("active".to_string()),
        tgi_start: Some(project_date.clone()),
        tgi_end: None,
        keterangan: format!("Progress Projek {} - Import from Excel: {}", 
            project_type_hint,
            file_name
        ),
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
    
    let project_id_clean = strip_table_prefix(&project_id_str, "projects");
    let project_thing = Thing::try_from(("projects", project_id_clean))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Parse sites (Row 1+ = index 1+ since calamine skips empty rows)
    let mut created_sites: Vec<Site> = Vec::new();
    let mut errors: Vec<ImportError> = Vec::new();
    
    for (idx, row) in rows.iter().enumerate().skip(1) {
        let row_number = idx + 1;
        
        // Get site_id to check if row has data
        let site_id = if let Some(col) = col_site_id {
            get_cell_string(row, col)
        } else {
            String::new()
        };
        
        if site_id.is_empty() {
            continue; // Skip empty rows
        }
        
        let site_name = if let Some(col) = col_site_name {
            let name = get_cell_string(row, col);
            if !name.is_empty() { name } else { site_id.clone() }
        } else {
            site_id.clone()
        };
        
        let lokasi = if let Some(col) = col_region {
            get_cell_string(row, col)
        } else {
            project_lokasi.clone()
        };
        
        let pekerjaan = if let Some(col) = col_pekerjaan {
            get_cell_string(row, col)
        } else {
            project_name.clone()
        };
        
        let nomor_kontrak = if let Some(col) = col_nop {
            let nop = get_cell_string(row, col);
            if !nop.is_empty() { nop } else { format!("PO-{}-{}", row_number, chrono::Utc::now().timestamp()) }
        } else {
            format!("PO-{}-{}", row_number, chrono::Utc::now().timestamp())
        };
        
        let maximal_budget = if let Some(col) = col_total_price {
            get_cell_i64(row, col)
        } else {
            0
        };
        
        let mitra = if let Some(col) = col_mitra {
            get_cell_string(row, col)
        } else {
            "Vendor/Pelaksana".to_string()
        };
        
        let site_info = format!("Site ID: {} | Project: {} | Imported from: {}", 
            site_id, project_type_hint, file_name);
        
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
            project_type = $project_type,
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
            .bind(("start", project_date.clone()))
            .bind(("end", project_date.clone()))
            .bind(("maximal_budget", maximal_budget))
            .bind(("cost_estimated", maximal_budget))
            .bind(("pemberi_tugas", "PT Telkom Indonesia".to_string()))
            .bind(("penerima_tugas", mitra))
            .bind(("site_document", None::<String>))
            .bind(("project_type", Some(project_type.clone())))
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
    
    let total_rows = rows.len() - 1; // Subtract header row (calamine skips empty rows)
    let summary = ImportSummary {
        project_id: project_id_str.clone(),
        project_name: project_name.clone(),
        total_budget,
        sites_count: created_sites.len(),
        message: format!(
            "EPROC Import: {} sites created, {} failed out of {} rows",
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
        message: Some("Bulk import from EPROC Excel completed successfully".to_string()),
    }))
}

// Parse old format (3 sheets, Sheet 3 "Active Project Details", Row 5=headers, Row 6+=data)
async fn parse_old_format(
    state: Arc<AppState>,
    mut workbook: Xlsx<Cursor<Vec<u8>>>,
    sheet_names: Vec<String>,
    file_name: String,
    project_lokasi: String,
    project_date: String,
    project_type_hint: &str,
) -> Result<Json<ApiResponse<BulkImportExcelResponse>>, StatusCode> {
    
    let target_sheet = &sheet_names[2];
    let range = workbook
        .worksheet_range(target_sheet)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Step 4: Get project info from Excel
    let rows: Vec<_> = range.rows().collect();
    
    // Get project budget from Row 2
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
    
    // Use project_type_hint if it's not the default "BEBAN OPERASIONAL"
    let project_type_str = if project_type_hint != "BEBAN OPERASIONAL" {
        project_type_hint.to_string()
    } else if rows.len() > 5 {
        get_cell_string(rows[5], 1) // Row 6, Column B
    } else {
        project_type_hint.to_string()
    };
    
    let project_type = parse_project_type(&project_type_str);
    
    // Create project name: "{TIPE} Project {LOKASI}"
    let project_name = format!("{} Project {}", 
        project_type_str.to_uppercase().trim(), 
        project_lokasi
    );
    
    // Step 5: Create Project
    let new_project = Project {
        id: None,
        name: project_name.clone(),
        lokasi: project_lokasi.clone(),
        value: total_boq_aktual,
        cost: total_nilai_po,
        tipe: project_type.clone(),
        status: Some("active".to_string()),
        tgi_start: Some(project_date.clone()),
        tgi_end: None,
        keterangan: format!("Progress Projek {} - Import from Excel: {}", 
            project_type_str.to_uppercase().trim(),
            file_name
        ),
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

        // Extract new fields for "site id based" view
        // C: SITE_ID
        let site_id = get_cell_string(row, 2);
        // E: SECTOR
        let sector = get_cell_string(row, 4);
        // F: CLUSTER
        let cluster = get_cell_string(row, 5);
        // Q: REGION (Assuming Column Q based on common formats, or fallback to empty)
        let region = get_cell_string(row, 16);
        
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
            project_type = $project_type,
            site_id = $site_id,
            sector = $sector,
            cluster = $cluster,
            region = $region,
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
            .bind(("project_type", Some(project_type.clone())))
            .bind(("site_id", site_id))
            .bind(("sector", sector))
            .bind(("cluster", cluster))
            .bind(("region", region))
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
