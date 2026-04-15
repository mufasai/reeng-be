use crate::extractors::FormOrJson;
use axum::{extract::{Json, Multipart, Path, State}, http::StatusCode};
use std::sync::Arc;
use std::io::Cursor;
use surrealdb::sql::Thing;
use calamine::{Reader, Xlsx, Data};

use crate::models::{ApiResponse, CreateTeamRequest, Team, TeamPeople, TeamUploadResult, UpdateTeamRequest};
use crate::state::AppState;

pub async fn create_team(
    State(state): State<Arc<AppState>>,
    FormOrJson(req): FormOrJson<CreateTeamRequest>,
) -> Result<Json<ApiResponse<Team>>, StatusCode> {
    // Helper function to strip table prefix if present
    fn strip_table_prefix<'a>(id_str: &'a str, table: &str) -> &'a str {
        let prefix = format!("{}:", table);
        id_str.strip_prefix(&prefix).unwrap_or(id_str)
    }

    // Parse project_id
    let project_id_clean = strip_table_prefix(&req.project_id, "projects");
    let project_thing = Thing::try_from(("projects", project_id_clean))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Parse site_id if provided
    let site_thing = if let Some(site_id) = &req.site_id {
        let site_id_clean = strip_table_prefix(site_id, "sites");
        Some(Thing::try_from(("sites", site_id_clean))
            .map_err(|_| StatusCode::BAD_REQUEST)?)
    } else {
        None
    };

    // Parse leader_id if provided
    let leader_thing = if let Some(leader_id) = &req.leader_id {
        let leader_id_clean = strip_table_prefix(leader_id, "people");
        Some(Thing::try_from(("people", leader_id_clean))
            .map_err(|_| StatusCode::BAD_REQUEST)?)
    } else {
        None
    };

    // Save team to database using raw query to avoid serialization issues
    let query = "CREATE teams SET nama = $nama, project_id = $project_id, site_id = $site_id, leader_id = $leader_id, active = $active, created_at = time::now(), updated_at = time::now()";
    
    let mut result = state.db.query(query)
        .bind(("nama", req.nama.clone()))
        .bind(("project_id", project_thing.clone()))
        .bind(("site_id", site_thing.clone()))
        .bind(("leader_id", leader_thing.clone()))
        .bind(("active", true))
        .await
        .map_err(|e| {
            eprintln!("Database error creating team: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let created_team: Option<Team> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let created_team = created_team.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let team_id = created_team.id.clone().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Add team members
    for member in req.members {
        let people_id_clean = strip_table_prefix(&member.people_id, "people");
        let people_thing = Thing::try_from(("people", people_id_clean))
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        // Use query instead of content to avoid serialization issues
        let member_query = "CREATE team_people SET team_id = $team_id, people_id = $people_id, role = $role, vendor = $vendor, created_at = time::now(), updated_at = time::now()";
        
        let _ = state.db.query(member_query)
            .bind(("team_id", team_id.clone()))
            .bind(("people_id", people_thing))
            .bind(("role", member.role))
            .bind(("vendor", member.vendor))
            .await
            .map_err(|e| {
                eprintln!("Database error creating team_people: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(created_team),
        message: Some("Team created successfully".to_string()),
    }))
}

pub async fn list_teams(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Team>>>, StatusCode> {
    // List all master teams (data master) ordered by creation date
    let query = "SELECT * FROM teams ORDER BY created_at DESC";
    let mut result = state.db.query(query)
        .await
        .map_err(|e| {
            eprintln!("Database error listing teams: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Try to parse teams, but if it fails, return empty array instead of error
    let teams: Vec<Team> = match result.take(0) {
        Ok(teams) => teams,
        Err(e) => {
            eprintln!("Parse error listing teams (returning empty): {}", e);
            Vec::new()  // Return empty array instead of failing
        }
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(teams),
        message: None,
    }))
}

pub async fn get_team(
    State(state): State<Arc<AppState>>,
    Path(team_id): Path<String>,
) -> Result<Json<ApiResponse<Team>>, StatusCode> {
    // Helper function to strip table prefix if present
    fn strip_table_prefix<'a>(id_str: &'a str, table: &str) -> &'a str {
        let prefix = format!("{}:", table);
        id_str.strip_prefix(&prefix).unwrap_or(id_str)
    }

    let team_id_clean = strip_table_prefix(&team_id, "teams");
    let thing = Thing::try_from(("teams", team_id_clean))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = "SELECT * FROM $team_id";
    let mut result = state.db.query(query)
        .bind(("team_id", thing))
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let team: Option<Team> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match team {
        Some(team) => Ok(Json(ApiResponse {
            success: true,
            data: Some(team),
            message: None,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_team_by_leader(
    State(state): State<Arc<AppState>>,
    Path(leader_id): Path<String>,
) -> Result<Json<ApiResponse<Team>>, StatusCode> {
    // Helper function to strip table prefix if present
    fn strip_table_prefix<'a>(id_str: &'a str, table: &str) -> &'a str {
        let prefix = format!("{}:", table);
        id_str.strip_prefix(&prefix).unwrap_or(id_str)
    }

    let leader_id_clean = strip_table_prefix(&leader_id, "people");
    let leader_thing = Thing::try_from(("people", leader_id_clean))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = "SELECT * FROM teams WHERE leader_id = $leader_id LIMIT 1";
    let mut result = state.db.query(query)
        .bind(("leader_id", leader_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let team: Option<Team> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match team {
        Some(team) => Ok(Json(ApiResponse {
            success: true,
            data: Some(team),
            message: None,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_team(
    State(state): State<Arc<AppState>>,
    Path(team_id): Path<String>,
    FormOrJson(req): FormOrJson<UpdateTeamRequest>,
) -> Result<Json<ApiResponse<Team>>, StatusCode> {
    // Helper function to strip table prefix if present
    fn strip_table_prefix<'a>(id_str: &'a str, table: &str) -> &'a str {
        let prefix = format!("{}:", table);
        id_str.strip_prefix(&prefix).unwrap_or(id_str)
    }

    let team_id_clean = strip_table_prefix(&team_id, "teams");
    let thing = Thing::try_from(("teams", team_id_clean))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Build UPDATE query dynamically based on provided fields
    let mut update_parts = Vec::new();
    let mut has_updates = false;
    
    if req.nama.is_some() {
        update_parts.push("nama = $nama");
        has_updates = true;
    }
    if req.project_id.is_some() {
        update_parts.push("project_id = $project_id");
        has_updates = true;
    }
    if req.site_id.is_some() {
        update_parts.push("site_id = $site_id");
        has_updates = true;
    }
    if req.leader_id.is_some() {
        update_parts.push("leader_id = $leader_id");
        has_updates = true;
    }
    if req.active.is_some() {
        update_parts.push("active = $active");
        has_updates = true;
    }

    if !has_updates {
        return Err(StatusCode::BAD_REQUEST);
    }

    update_parts.push("updated_at = time::now()");

    let query = format!(
        "UPDATE $team_id SET {}",
        update_parts.join(", ")
    );

    let mut query_builder = state.db.query(&query)
        .bind(("team_id", thing));

    if let Some(nama) = req.nama {
        query_builder = query_builder.bind(("nama", nama));
    }
    if let Some(project_id) = req.project_id {
        let project_id_clean = strip_table_prefix(&project_id, "projects");
        let project_thing = Thing::try_from(("projects", project_id_clean))
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        query_builder = query_builder.bind(("project_id", project_thing));
    }
    if let Some(site_id) = req.site_id {
        let site_id_clean = strip_table_prefix(&site_id, "sites");
        let site_thing = Thing::try_from(("sites", site_id_clean))
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        query_builder = query_builder.bind(("site_id", site_thing));
    }
    if let Some(leader_id) = req.leader_id {
        let leader_id_clean = strip_table_prefix(&leader_id, "people");
        let leader_thing = Thing::try_from(("people", leader_id_clean))
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        query_builder = query_builder.bind(("leader_id", leader_thing));
    }
    if let Some(active) = req.active {
        query_builder = query_builder.bind(("active", active));
    }

    let mut result = query_builder.await
        .map_err(|e| {
            eprintln!("Database error updating team: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let team: Option<Team> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match team {
        Some(team) => Ok(Json(ApiResponse {
            success: true,
            data: Some(team),
            message: Some("Team updated successfully".to_string()),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_team(
    State(state): State<Arc<AppState>>,
    Path(team_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // Helper function to strip table prefix if present
    fn strip_table_prefix<'a>(id_str: &'a str, table: &str) -> &'a str {
        let prefix = format!("{}:", table);
        id_str.strip_prefix(&prefix).unwrap_or(id_str)
    }

    let team_id_clean = strip_table_prefix(&team_id, "teams");
    let thing = Thing::try_from(("teams", team_id_clean))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // First, delete all team members
    let delete_members = "DELETE team_people WHERE team_id = $team_id";
    let _ = state.db.query(delete_members)
        .bind(("team_id", thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting team members: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Then delete the team
    let delete_team_query = "DELETE $team_id";
    let _ = state.db.query(delete_team_query)
        .bind(("team_id", thing))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting team: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("Team deleted successfully".to_string()),
    }))
}

pub async fn list_team_members(
    State(state): State<Arc<AppState>>,
    Path(team_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<TeamPeople>>>, StatusCode> {
    // Helper function to strip table prefix if present
    fn strip_table_prefix<'a>(id_str: &'a str, table: &str) -> &'a str {
        let prefix = format!("{}:", table);
        id_str.strip_prefix(&prefix).unwrap_or(id_str)
    }

    let team_id_clean = strip_table_prefix(&team_id, "teams");
    let thing = Thing::try_from(("teams", team_id_clean))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = "SELECT * FROM team_people WHERE team_id = $team_id";
    
    let mut result = state.db.query(query)
        .bind(("team_id", thing))
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let members: Vec<TeamPeople> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(members),
        message: None,
    }))
}

pub async fn upload_teams_excel(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<TeamUploadResult>>, StatusCode> {
    // Extract file from multipart form
    let mut file_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            file_data = Some(bytes.to_vec());
        }
    }

    let file_bytes = file_data.ok_or_else(|| {
        eprintln!("No file field found in multipart form");
        StatusCode::BAD_REQUEST
    })?;

    // Parse Excel file from memory
    let cursor = Cursor::new(file_bytes);
    let mut workbook: Xlsx<_> = Xlsx::new(cursor).map_err(|e| {
        eprintln!("Failed to parse Excel file: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Get the first sheet name
    let sheet_names = workbook.sheet_names().to_vec();
    let sheet_name = sheet_names.first().ok_or_else(|| {
        eprintln!("Excel file has no sheets");
        StatusCode::BAD_REQUEST
    })?;

    let range = workbook.worksheet_range(sheet_name).map_err(|e| {
        eprintln!("Failed to read worksheet: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    let rows: Vec<Vec<Data>> = range.rows().map(|r| r.to_vec()).collect();

    if rows.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Parse header row to determine column mapping
    let header_row_idx = rows.iter().position(|row| {
        row.iter().any(|cell| {
            let val = cell_to_string(cell).to_lowercase();
            let val = val.trim();
            val == "nik" || val == "nama_karyawan" || val == "nama karyawan"
        })
    }).ok_or_else(|| {
        eprintln!("Could not find header row in Excel file");
        StatusCode::BAD_REQUEST
    })?;

    let headers: Vec<String> = rows[header_row_idx]
        .iter()
        .map(|cell| {
            cell_to_string(cell)
                .to_lowercase()
                .trim()
                .replace(" ", "_")
                .to_string()
        })
        .collect();

    let mut total_rows = 0usize;
    let mut success_count = 0usize;
    let mut failed_count = 0usize;
    let mut errors: Vec<String> = Vec::new();

    // Process data rows (skip rows up to and including header)
    for (row_idx, row) in rows.iter().skip(header_row_idx + 1).enumerate() {
        let row_num = header_row_idx + row_idx + 2;
        total_rows += 1;

        // Map columns by header name
        let get_col = |name: &str| -> Option<String> {
            headers.iter().position(|h| h == name).and_then(|idx| {
                row.get(idx).map(|cell| {
                    let val = cell_to_string(cell);
                    if val.is_empty() { return String::new(); }
                    val
                })
            })
        };

        let nik = get_col("nik");
        let nama_karyawan = get_col("nama_karyawan");
        let tanggal_lahir = headers.iter().position(|h| h == "tanggal_lahir").and_then(|idx| {
            row.get(idx).map(|cell| match cell {
                Data::Float(f) => {
                    let s = excel_serial_to_date(*f);
                    if s.is_empty() { None } else { Some(s) }
                }
                Data::Int(i) => {
                    let s = excel_serial_to_date(*i as f64);
                    if s.is_empty() { None } else { Some(s) }
                }
                Data::DateTime(dt) => {
                    let serial = dt.as_f64();
                    let s = excel_serial_to_date(serial);
                    if s.is_empty() { None } else { Some(s) }
                }
                Data::String(s) if !s.is_empty() => Some(s.clone()),
                _ => None,
            })
        }).flatten();
        let tempat_lahir = get_col("tempat_lahir");
        let agama = get_col("agama");
        let jenis_kelamin = get_col("jenis_kelamin");
        let no_ktp = get_col("no_ktp");
        let no_hp = get_col("no_hp");
        let alamat_email = get_col("alamat_email");
        let jabatan_kerja = get_col("jabatan_kerja");
        let regional = get_col("regional");

        // Use nama_karyawan as the team "nama" field, or fallback to nik
        let nama = nama_karyawan.clone()
            .filter(|s| !s.is_empty())
            .or_else(|| nik.clone().filter(|s| !s.is_empty()))
            .unwrap_or_else(|| format!("Row {}", row_num));

        // Insert into database
        let query = "CREATE teams SET \
            nama = $nama, \
            nik = $nik, \
            nama_karyawan = $nama_karyawan, \
            tanggal_lahir = <option<datetime>> $tanggal_lahir, \
            tempat_lahir = $tempat_lahir, \
            agama = $agama, \
            jenis_kelamin = $jenis_kelamin, \
            no_ktp = $no_ktp, \
            no_hp = $no_hp, \
            alamat_email = $alamat_email, \
            jabatan_kerja = $jabatan_kerja, \
            regional = $regional, \
            active = true, \
            created_at = time::now(), \
            updated_at = time::now()";

        let mut result = state.db.query(query)
            .bind(("nama", nama))
            .bind(("nik", nik))
            .bind(("nama_karyawan", nama_karyawan))
            .bind(("tanggal_lahir", tanggal_lahir))
            .bind(("tempat_lahir", tempat_lahir))
            .bind(("agama", agama))
            .bind(("jenis_kelamin", jenis_kelamin))
            .bind(("no_ktp", no_ktp))
            .bind(("no_hp", no_hp))
            .bind(("alamat_email", alamat_email))
            .bind(("jabatan_kerja", jabatan_kerja))
            .bind(("regional", regional))
            .await;

        match result {
            Ok(ref mut res) => {
                // Take the result to ensure the query is executed
                match res.take::<Option<Team>>(0) {
                    Ok(Some(_)) => {
                        success_count += 1;
                    }
                    Ok(None) => {
                        failed_count += 1;
                        errors.push(format!("Row {}: No data returned from database", row_num));
                        eprintln!("Error inserting team row {}: No data returned", row_num);
                    }
                    Err(e) => {
                        failed_count += 1;
                        errors.push(format!("Row {}: {}", row_num, e));
                        eprintln!("Error inserting team row {}: {}", row_num, e);
                    }
                }
            }
            Err(e) => {
                failed_count += 1;
                errors.push(format!("Row {}: {}", row_num, e));
                eprintln!("Error inserting team row {}: {}", row_num, e);
            }
        }
    }

    let upload_result = TeamUploadResult {
        total_rows,
        success_count,
        failed_count,
        errors,
    };

    Ok(Json(ApiResponse {
        success: true,
        data: Some(upload_result),
        message: Some(format!("{} of {} teams imported successfully", success_count, total_rows)),
    }))
}

/// Helper: convert a calamine Data cell to a String
fn cell_to_string(data: &Data) -> String {
    match data {
        Data::String(s) => s.clone(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            // If the float is a whole number, format without decimals
            if *f == (*f as i64) as f64 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => {
            // Use Display trait to format ExcelDateTime
            format!("{}", dt)
        }
        Data::Error(e) => format!("ERROR({:?})", e),
        Data::Empty => String::new(),
        _ => String::new(),
    }
}

fn excel_serial_to_date(serial: f64) -> String {
    let days = if serial >= 60.0 { serial as i64 - 1 } else { serial as i64 };
    let base = chrono::NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    match base.checked_add_days(chrono::Days::new(days as u64)) {
        Some(date) => date.format("%Y-%m-%dT00:00:00Z").to_string(),
        None => String::new(),
    }
}