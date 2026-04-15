use crate::extractors::FormOrJson;
use axum::{extract::{Json, Path, Multipart, State}, http::StatusCode};
use std::sync::Arc;
use std::io::Cursor;
use calamine::{Reader, Xlsx, Data};

use crate::models::{ApiResponse, CreatePeopleRequest, UpdatePeopleRequest, People, TeamUploadResult};
use crate::state::AppState;

pub async fn create_people(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    FormOrJson(req): FormOrJson<CreatePeopleRequest>,
) -> Result<Json<ApiResponse<People>>, StatusCode> {
    let people = People {
        id: None,
        name: req.name,
        tanggal_lahir: req.tanggal_lahir,
        tempat_lahir: req.tempat_lahir,
        agama: req.agama,
        jenis_kelamin: req.jenis_kelamin,
        no_ktp: req.no_ktp,
        no_hp: req.no_hp,
        email: req.email,
        jabatan_kerja: req.jabatan_kerja,
        regional: req.regional,
        lokasi_kerja: req.lokasi_kerja,
        pekerjaan: req.pekerjaan,
        nama_kontak_darurat: None,
        nomor_kontak_darurat: None,
        alamat_kontak_darurat: None,
        status_pernikahan: None,
        nama_ibu_kandung: None,
        pendidikan_terakhir: None,
        nama_kampus_sekolah: None,
        jurusan_sekolah: None,
        tahun_lulus: None,
        foto_ktp: None,
        foto_diri: None,
        thumbnail_path: None,
        created_at: None,
        updated_at: None,
    };

    let created: Option<People> = state
        .db
        .create("people")
        .content(people.clone())
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(created.unwrap_or(people)),
        message: Some("Person created successfully".to_string()),
    }))
}

pub async fn list_people(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<People>>>, StatusCode> {
    let mut response = state
        .db
        .query("SELECT * FROM people ORDER BY name ASC")
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let people: Vec<People> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(people),
        message: None,
    }))
}

pub async fn get_people(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Path(people_id): Path<String>,
) -> Result<Json<ApiResponse<People>>, StatusCode> {
    let query = "SELECT * FROM type::thing('people', $id)";

    let mut response = state
        .db
        .query(query)
        .bind(("id", people_id.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let people: Vec<People> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if people.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(people[0].clone()),
        message: None,
    }))
}

pub async fn update_people(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Path(people_id): Path<String>,
    FormOrJson(req): FormOrJson<UpdatePeopleRequest>,
) -> Result<Json<ApiResponse<People>>, StatusCode> {
    // Get existing person
    let query = "SELECT * FROM type::thing('people', $id)";
    let mut response = state
        .db
        .query(query)
        .bind(("id", people_id.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let people: Vec<People> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if people.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut existing_person = people[0].clone();

    // Update only provided fields
    if let Some(name) = req.name {
        existing_person.name = name;
    }
    if let Some(tanggal_lahir) = req.tanggal_lahir {
        existing_person.tanggal_lahir = Some(tanggal_lahir);
    }
    if let Some(tempat_lahir) = req.tempat_lahir {
        existing_person.tempat_lahir = Some(tempat_lahir);
    }
    if let Some(agama) = req.agama {
        existing_person.agama = Some(agama);
    }
    if let Some(jenis_kelamin) = req.jenis_kelamin {
        existing_person.jenis_kelamin = Some(jenis_kelamin);
    }
    if let Some(no_ktp) = req.no_ktp {
        existing_person.no_ktp = Some(no_ktp);
    }
    if let Some(no_hp) = req.no_hp {
        existing_person.no_hp = Some(no_hp);
    }
    if let Some(email) = req.email {
        existing_person.email = Some(email);
    }
    if let Some(jabatan_kerja) = req.jabatan_kerja {
        existing_person.jabatan_kerja = Some(jabatan_kerja);
    }
    if let Some(regional) = req.regional {
        existing_person.regional = Some(regional);
    }
    if let Some(lokasi_kerja) = req.lokasi_kerja {
        existing_person.lokasi_kerja = Some(lokasi_kerja);
    }
    if let Some(pekerjaan) = req.pekerjaan {
        existing_person.pekerjaan = Some(pekerjaan);
    }
    if let Some(nama_kontak_darurat) = req.nama_kontak_darurat {
        existing_person.nama_kontak_darurat = Some(nama_kontak_darurat);
    }
    if let Some(nomor_kontak_darurat) = req.nomor_kontak_darurat {
        existing_person.nomor_kontak_darurat = Some(nomor_kontak_darurat);
    }
    if let Some(alamat_kontak_darurat) = req.alamat_kontak_darurat {
        existing_person.alamat_kontak_darurat = Some(alamat_kontak_darurat);
    }
    if let Some(status_pernikahan) = req.status_pernikahan {
        existing_person.status_pernikahan = Some(status_pernikahan);
    }
    if let Some(nama_ibu_kandung) = req.nama_ibu_kandung {
        existing_person.nama_ibu_kandung = Some(nama_ibu_kandung);
    }
    if let Some(pendidikan_terakhir) = req.pendidikan_terakhir {
        existing_person.pendidikan_terakhir = Some(pendidikan_terakhir);
    }
    if let Some(nama_kampus_sekolah) = req.nama_kampus_sekolah {
        existing_person.nama_kampus_sekolah = Some(nama_kampus_sekolah);
    }
    if let Some(jurusan_sekolah) = req.jurusan_sekolah {
        existing_person.jurusan_sekolah = Some(jurusan_sekolah);
    }
    if let Some(tahun_lulus) = req.tahun_lulus {
        existing_person.tahun_lulus = Some(tahun_lulus);
    }

    // Update in database
    let update_query = r#"
        UPDATE type::thing('people', $id) SET
            name = $name,
            tanggal_lahir = $tanggal_lahir,
            tempat_lahir = $tempat_lahir,
            agama = $agama,
            jenis_kelamin = $jenis_kelamin,
            no_ktp = $no_ktp,
            no_hp = $no_hp,
            email = $email,
            jabatan_kerja = $jabatan_kerja,
            regional = $regional,
            lokasi_kerja = $lokasi_kerja,
            pekerjaan = $pekerjaan,
            nama_kontak_darurat = $nama_kontak_darurat,
            nomor_kontak_darurat = $nomor_kontak_darurat,
            alamat_kontak_darurat = $alamat_kontak_darurat,
            status_pernikahan = $status_pernikahan,
            nama_ibu_kandung = $nama_ibu_kandung,
            pendidikan_terakhir = $pendidikan_terakhir,
            nama_kampus_sekolah = $nama_kampus_sekolah,
            jurusan_sekolah = $jurusan_sekolah,
            tahun_lulus = $tahun_lulus,
            updated_at = time::now()
    "#;

    let mut update_response = state
        .db
        .query(update_query)
        .bind(("id", people_id.clone()))
        .bind(("name", existing_person.name.clone()))
        .bind(("tanggal_lahir", existing_person.tanggal_lahir.clone()))
        .bind(("tempat_lahir", existing_person.tempat_lahir.clone()))
        .bind(("agama", existing_person.agama.clone()))
        .bind(("jenis_kelamin", existing_person.jenis_kelamin.clone()))
        .bind(("no_ktp", existing_person.no_ktp.clone()))
        .bind(("no_hp", existing_person.no_hp.clone()))
        .bind(("email", existing_person.email.clone()))
        .bind(("jabatan_kerja", existing_person.jabatan_kerja.clone()))
        .bind(("regional", existing_person.regional.clone()))
        .bind(("lokasi_kerja", existing_person.lokasi_kerja.clone()))
        .bind(("pekerjaan", existing_person.pekerjaan.clone()))
        .bind(("nama_kontak_darurat", existing_person.nama_kontak_darurat.clone()))
        .bind(("nomor_kontak_darurat", existing_person.nomor_kontak_darurat.clone()))
        .bind(("alamat_kontak_darurat", existing_person.alamat_kontak_darurat.clone()))
        .bind(("status_pernikahan", existing_person.status_pernikahan.clone()))
        .bind(("nama_ibu_kandung", existing_person.nama_ibu_kandung.clone()))
        .bind(("pendidikan_terakhir", existing_person.pendidikan_terakhir.clone()))
        .bind(("nama_kampus_sekolah", existing_person.nama_kampus_sekolah.clone()))
        .bind(("jurusan_sekolah", existing_person.jurusan_sekolah.clone()))
        .bind(("tahun_lulus", existing_person.tahun_lulus.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let updated_people: Vec<People> = update_response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let updated_person = updated_people.first().cloned().unwrap_or(existing_person);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(updated_person),
        message: Some("Person updated successfully".to_string()),
    }))
}

pub async fn delete_people(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Path(people_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let query = "DELETE type::thing('people', $id)";

    let _result = state
        .db
        .query(query)
        .bind(("id", people_id.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(format!("Person {} deleted successfully", people_id)),
        message: Some("Person deleted successfully".to_string()),
    }))
}

// ==================== BULK IMPORT PEOPLE FROM EXCEL ====================

pub async fn upload_people_excel(
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
            val == "name" || val == "nama" || val == "nama_karyawan"
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

        // Handle date columns separately
        let get_date_col = |name: &str| -> Option<String> {
            headers.iter().position(|h| h == name).and_then(|idx| {
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
            }).flatten()
        };

        let name = get_col("name")
            .filter(|s| !s.is_empty())
            .or_else(|| get_col("nama").filter(|s| !s.is_empty()))
            .or_else(|| get_col("nama_karyawan").filter(|s| !s.is_empty()))
            .unwrap_or_else(|| format!("Row {}", row_num));

        let tanggal_lahir = get_date_col("tanggal_lahir");
        let tempat_lahir = get_col("tempat_lahir");
        let agama = get_col("agama");
        let jenis_kelamin = get_col("jenis_kelamin");
        let no_ktp = get_col("no_ktp");
        let no_hp = get_col("no_hp");
        let email = get_col("email").or_else(|| get_col("alamat_email"));
        let jabatan_kerja = get_col("jabatan_kerja");
        let regional = get_col("regional");
        let lokasi_kerja = get_col("lokasi_kerja");
        let pekerjaan = get_col("pekerjaan");

        // Insert into database
        let query = "CREATE people SET \
            name = $name, \
            tanggal_lahir = <option<datetime>> $tanggal_lahir, \
            tempat_lahir = $tempat_lahir, \
            agama = $agama, \
            jenis_kelamin = $jenis_kelamin, \
            no_ktp = $no_ktp, \
            no_hp = $no_hp, \
            email = $email, \
            jabatan_kerja = $jabatan_kerja, \
            regional = $regional, \
            lokasi_kerja = $lokasi_kerja, \
            pekerjaan = $pekerjaan, \
            created_at = time::now(), \
            updated_at = time::now()";

        let mut result = state.db.query(query)
            .bind(("name", name))
            .bind(("tanggal_lahir", tanggal_lahir))
            .bind(("tempat_lahir", tempat_lahir))
            .bind(("agama", agama))
            .bind(("jenis_kelamin", jenis_kelamin))
            .bind(("no_ktp", no_ktp))
            .bind(("no_hp", no_hp))
            .bind(("email", email))
            .bind(("jabatan_kerja", jabatan_kerja))
            .bind(("regional", regional))
            .bind(("lokasi_kerja", lokasi_kerja))
            .bind(("pekerjaan", pekerjaan))
            .await;

        match result {
            Ok(ref mut res) => {
                // Take the result to ensure the query is executed
                match res.take::<Option<People>>(0) {
                    Ok(Some(_)) => {
                        success_count += 1;
                    }
                    Ok(None) => {
                        failed_count += 1;
                        errors.push(format!("Row {}: No data returned from database", row_num));
                        eprintln!("Error inserting people row {}: No data returned", row_num);
                    }
                    Err(e) => {
                        failed_count += 1;
                        errors.push(format!("Row {}: {}", row_num, e));
                        eprintln!("Error inserting people row {}: {}", row_num, e);
                    }
                }
            }
            Err(e) => {
                failed_count += 1;
                errors.push(format!("Row {}: {}", row_num, e));
                eprintln!("Error inserting people row {}: {}", row_num, e);
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
        message: Some(format!("{} of {} people imported successfully", success_count, total_rows)),
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
