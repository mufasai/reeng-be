use axum::{extract::{Json, Path}, http::StatusCode};
use std::sync::Arc;

use crate::models::{ApiResponse, CreatePeopleRequest, UpdatePeopleRequest, People};
use crate::state::AppState;

pub async fn create_people(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<CreatePeopleRequest>,
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
    Json(req): Json<UpdatePeopleRequest>,
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
