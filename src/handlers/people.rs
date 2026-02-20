use axum::{extract::Json, http::StatusCode};
use std::sync::Arc;

use crate::models::{ApiResponse, CreatePeopleRequest, People};
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
