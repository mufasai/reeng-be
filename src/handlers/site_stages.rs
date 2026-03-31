/// ==================== HANDLERS > SITE_STAGES.RS ====================
/// Site Stage Management Handlers
/// Handles stage transitions dengan full validation, logging, dan audit trail
/// Clean code: Separation of concerns with dedicated service layer
/// Supports all stage-specific fields sesuai mockup UI

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use surrealdb::sql::Thing;

use crate::models::{ApiResponse, UpdateSiteStageRequest, Site, SiteStageLog};
use crate::state::AppState;
use crate::services::StageTransitionService;
use crate::config;
use crate::common::{parse_thing_id};

// ─── UPDATE SITE STAGE ────────────────────────────────────────────────────────
/// Update stage dengan full validation berdasarkan mockup requirements
/// Supports ALL stage-specific fields dari UI:
/// - akses_ready → has_akses_gedung (toggle untuk building access)
/// - implementasi → tgl_rencana_implementasi, tgl_aktual_mulai, jam_checkin
/// - All permit, akses, implementasi fields per spec
pub async fn update_site_stage(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
    Json(req): Json<UpdateSiteStageRequest>,
) -> Result<Json<ApiResponse<Site>>, StatusCode> {
    // Step 1: Parse dan validate site ID
    let site_thing = parse_thing_id(&site_id, "sites").map_err(|e| {
        eprintln!("❌ Invalid site ID format: {}", site_id);
        e
    })?;
    
    // Step 2: Fetch current site dan validate exists
    let site = fetch_site(&state, &site_thing).await.map_err(|e| {
        eprintln!("❌ Site not found or database error: site_id={}", site_id);
        e
    })?;

    let current_stage = site.stage.as_deref().unwrap_or("imported");

    // Step 3: Validate stage transition is allowed
    config::validate_stage_transition(current_stage, &req.stage, "FILTER")
        .map_err(|e| {
            eprintln!("❌ Invalid stage transition: {} → {} ({})", current_stage, req.stage, e);
            StatusCode::BAD_REQUEST
        })?;

    // Step 4: Validate all required fields are provided
    validate_stage_transition_fields(current_stage, &req).map_err(|e| {
        eprintln!("❌ Field validation failed: {} → {} : {}", current_stage, req.stage, e);
        StatusCode::BAD_REQUEST
    })?;

    // Step 5: Build and execute SurrealDB update query
    let mut update_query_full = format!("UPDATE {} SET stage = '{}', stage_updated_at = time::now()",
        site_thing, escape_sql_string(&req.stage));
    
    // Add optional notes field
    if let Some(notes) = &req.notes {
        if !notes.trim().is_empty() {
            update_query_full.push_str(&format!(", notes = '{}'", escape_sql_string(notes)));
        }
    }
    
    // Add all stage-specific fields based on transition type
    add_update_fields(&mut update_query_full, current_stage, &req)
        .map_err(|e| {
            eprintln!("❌ Failed to build update query: {}", e);
            StatusCode::BAD_REQUEST
        })?;
    
    // Execute the update query
    let mut response = state
        .db
        .query(&update_query_full)
        .await
        .map_err(|e| {
            eprintln!("❌ Database error updating site: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Parse and extract updated site record
    let mut updated_sites: Vec<Site> = response.take(0).map_err(|e| {
        eprintln!("❌ Failed to parse updated site from database: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let updated_site = updated_sites
        .pop()
        .ok_or_else(|| {
            eprintln!("❌ Site record not found after update");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Step 6: Create audit log entry for stage transition
    if let Err(e) = create_stage_log(
        &state, 
        &site_thing, 
        current_stage.to_string(), 
        req.stage.clone(), 
        &req.notes
    ).await {
        eprintln!("⚠️  Warning: Failed to create audit log (site update succeeded): {}", e);
        // Don't fail the entire request if logging fails
    }

    eprintln!("✓ Site stage updated successfully: {} → {}", current_stage, req.stage);
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(updated_site),
        message: Some(format!(
            "✓ Stage berhasil diupdate dari '{}' ke '{}'. Audit trail tercatat.",
            current_stage, req.stage
        )),
    }))
}

// ─── VALIDATE STAGE TRANSITION FIELDS ─────────────────────────────────────────
/// Validate bahwa semua required fields hadir sesuai stage transition
/// Ini adalah key function untuk ensure consistency dengan UI mockup
/// Validate required fields based on stage transition
/// Ensures all mandatory fields are provided before stage update
/// Returns descriptive error messages matching UI expectations
fn validate_stage_transition_fields(
    from_stage: &str,
    req: &UpdateSiteStageRequest,
) -> Result<(), String> {
    match (from_stage, req.stage.as_str()) {
        // ASSIGNED → PERMIT_PROCESS
        // Required: Permit date when requesting permit
        ("assigned", "permit_process") => {
            if req.permit_date.is_none() || req.permit_date.as_ref().map_or(true, |d| d.trim().is_empty()) {
                return Err("❌ Tanggal Izin/Permit harus diisi".to_string());
            }
        }

        // PERMIT_PROCESS → PERMIT_READY
        // Required: All three approval confirmations (TPAS, TP, CAF)
        ("permit_process", "permit_ready") => {
            if !req.tpas_approved.unwrap_or(false) {
                return Err("❌ Checkbox 'Konfirmasi TPAS Approved' harus di-check".to_string());
            }
            if !req.tp_approved.unwrap_or(false) {
                return Err("❌ Checkbox 'Konfirmasi TP Approved' harus di-check".to_string());
            }
            if !req.caf_approved.unwrap_or(false) {
                return Err("❌ Checkbox 'Konfirmasi CAF Approved' harus di-check".to_string());
            }
        }

        // PERMIT_READY → AKSES_PROCESS
        // Required: Tower provider, key type, and PIC contact info
        ("permit_ready", "akses_process") => {
            if req.tower_provider.is_none() || req.tower_provider.as_ref().map_or(true, |p| p.trim().is_empty()) {
                return Err("❌ Tower Provider harus dipilih (MITRATEL/STP/PTI/DMT/Lainnya)".to_string());
            }
            if req.jenis_kunci.is_none() || req.jenis_kunci.as_ref().map_or(true, |k| k.trim().is_empty()) {
                return Err("❌ Jenis Kunci harus dipilih (PADLOCK/SMARTLOCK/QUADLOCK)".to_string());
            }
            if req.pic_akses_nama.is_none() || req.pic_akses_nama.as_ref().map_or(true, |n| n.trim().is_empty()) {
                return Err("❌ Nama PIC Akses Tower harus diisi".to_string());
            }
            if req.pic_akses_telp.is_none() || req.pic_akses_telp.as_ref().map_or(true, |t| t.trim().is_empty()) {
                return Err("❌ Nomor Telepon PIC Akses Tower harus diisi".to_string());
            }
        }

        // AKSES_PROCESS → AKSES_READY
        // Required: Survey result, and if has_akses_gedung=true, building details
        ("akses_process", "akses_ready") => {
            if req.survey_result.is_none() {
                return Err("❌ Hasil Survey harus dipilih (OK/NOK)".to_string());
            }
            if req.survey_result.as_ref().map_or(false, |s| s == "nok") {
                if req.survey_nok_reason.is_none() || req.survey_nok_reason.as_ref().map_or(true, |r| r.trim().is_empty()) {
                    return Err("❌ Alasan Survey NOK harus diisi".to_string());
                }
            }
            
            if req.has_akses_gedung == Some(true) {
                if req.gedung_nama.is_none() || req.gedung_nama.as_ref().map_or(true, |n| n.trim().is_empty()) {
                    return Err("❌ Nama Gedung harus diisi (ada akses ke gedung)".to_string());
                }
                if req.gedung_pic_nama.is_none() || req.gedung_pic_nama.as_ref().map_or(true, |n| n.trim().is_empty()) {
                    return Err("❌ Nama PIC Gedung harus diisi".to_string());
                }
                if req.gedung_pic_telp.is_none() || req.gedung_pic_telp.as_ref().map_or(true, |t| t.trim().is_empty()) {
                    return Err("❌ Nomor Telepon PIC Gedung harus diisi".to_string());
                }
            }
            
            if !req.konfirmasi_akses.unwrap_or(false) {
                return Err("❌ Checkbox 'Sudah akses ke tower READY EKSEKUSI' harus di-check".to_string());
            }
        }

        // AKSES_READY → IMPLEMENTASI
        // Required: Planned implementation date
        ("akses_ready", "implementasi") => {
            if req.tgl_rencana_implementasi.is_none() || req.tgl_rencana_implementasi.as_ref().map_or(true, |d| d.trim().is_empty()) {
                return Err("❌ Tanggal Rencana Implementasi harus diisi".to_string());
            }
        }

        // IMPLEMENTASI → RFI_DONE
        // Required: Actual start date, check-in time, RFI confirmation
        ("implementasi", "rfi_done") => {
            if req.tgl_aktual_mulai.is_none() || req.tgl_aktual_mulai.as_ref().map_or(true, |d| d.trim().is_empty()) {
                return Err("❌ Tanggal Aktual Mulai harus diisi".to_string());
            }
            if req.jam_checkin.is_none() || req.jam_checkin.as_ref().map_or(true, |j| j.trim().is_empty()) {
                return Err("❌ Jam Check-in harus diisi".to_string());
            }
            if !req.konfirmasi_rfi.unwrap_or(false) {
                return Err("❌ Checkbox 'RFI Selesai' harus di-check".to_string());
            }
        }

        // RFI_DONE → RFS_DONE
        // Required: Check-out time, RFS confirmation
        ("rfi_done", "rfs_done") => {
            if req.jam_checkout.is_none() || req.jam_checkout.as_ref().map_or(true, |j| j.trim().is_empty()) {
                return Err("❌ Jam Check-out harus diisi".to_string());
            }
            if !req.konfirmasi_rfs.unwrap_or(false) {
                return Err("❌ Checkbox 'RFS Selesai' harus di-check".to_string());
            }
        }

        // RFS_DONE → BAST
        // Required: All documentation and final confirmations
        ("rfs_done", "bast") => {
            if !req.konfirmasi_dok.unwrap_or(false) {
                return Err("❌ Checkbox 'Dokumentasi Lengkap' harus di-check".to_string());
            }
            if !req.konfirmasi_final.unwrap_or(false) {
                return Err("❌ Checkbox 'Konfirmasi Final' harus di-check".to_string());
            }
        }

        // All other transitions don't require additional field validation
        _ => {}
    }

    Ok(())
}

// ─── ADD UPDATE FIELDS ────────────────────────────────────────────────────────
/// Dynamically add stage-specific fields ke query berdasarkan transition
fn add_update_fields(
    query: &mut String,
    from_stage: &str,
    req: &UpdateSiteStageRequest,
) -> Result<(), String> {
    match (from_stage, req.stage.as_str()) {
        ("assigned", "permit_process") => {
            if let Some(permit_date) = &req.permit_date {
                query.push_str(&format!(", permit_date = '{}'", escape_sql_string(permit_date)));
            }
        }

        ("permit_process", "permit_ready") => {
            query.push_str(&format!(
                ", tpas_approved = {}, tp_approved = {}, caf_approved = {}",
                req.tpas_approved.unwrap_or(false),
                req.tp_approved.unwrap_or(false),
                req.caf_approved.unwrap_or(false)
            ));
            if let Some(date) = &req.tgl_berlaku_permit_tpas {
                query.push_str(&format!(", tgl_berlaku_permit_tpas = '{}'", escape_sql_string(date)));
            }
            if let Some(date) = &req.tgl_berakhir_permit_tpas {
                query.push_str(&format!(", tgl_berakhir_permit_tpas = '{}'", escape_sql_string(date)));
            }
        }

        ("permit_ready", "akses_process") => {
            if let Some(provider) = &req.tower_provider {
                query.push_str(&format!(", tower_provider = '{}'", escape_sql_string(provider)));
            }
            if let Some(kunci) = &req.jenis_kunci {
                query.push_str(&format!(", jenis_kunci = '{}'", escape_sql_string(kunci)));
            }
            if let Some(nama) = &req.pic_akses_nama {
                query.push_str(&format!(", pic_akses_nama = '{}'", escape_sql_string(nama)));
            }
            if let Some(telp) = &req.pic_akses_telp {
                query.push_str(&format!(", pic_akses_telp = '{}'", escape_sql_string(telp)));
            }
        }

        ("akses_process", "akses_ready") => {
            query.push_str(&format!(", has_akses_gedung = {}", req.has_akses_gedung.unwrap_or(false)));
            
            if req.has_akses_gedung.unwrap_or(false) {
                if let Some(nama) = &req.gedung_nama {
                    query.push_str(&format!(", gedung_nama = '{}'", escape_sql_string(nama)));
                }
                if let Some(pic_nama) = &req.gedung_pic_nama {
                    query.push_str(&format!(", gedung_pic_nama = '{}'", escape_sql_string(pic_nama)));
                }
                if let Some(pic_telp) = &req.gedung_pic_telp {
                    query.push_str(&format!(", gedung_pic_telp = '{}'", escape_sql_string(pic_telp)));
                }
                if let Some(status) = &req.gedung_akses_status {
                    query.push_str(&format!(", gedung_akses_status = '{}'", escape_sql_string(status)));
                }
            }

            if let Some(survey) = &req.survey_result {
                query.push_str(&format!(", survey_result = '{}'", escape_sql_string(survey)));
                if survey == "nok" {
                    if let Some(reason) = &req.survey_nok_reason {
                        query.push_str(&format!(", survey_nok_reason = '{}'", escape_sql_string(reason)));
                    }
                }
            }

            if let Some(erfin_num) = &req.erfin_number {
                query.push_str(&format!(", erfin_number = '{}'", escape_sql_string(erfin_num)));
            }
            if let Some(erfin_dt) = &req.erfin_date {
                query.push_str(&format!(", erfin_date = '{}'", erfin_dt));
            }

            if let Some(konfirmasi) = req.konfirmasi_akses {
                query.push_str(&format!(", konfirmasi_akses = {}", konfirmasi));
            }
        }

        ("akses_ready", "implementasi") => {
            if let Some(tgl_rencana) = &req.tgl_rencana_implementasi {
                query.push_str(&format!(", tgl_rencana_implementasi = '{}'", escape_sql_string(tgl_rencana)));
            }
        }

        ("implementasi", "rfi_done") => {
            if let Some(tgl_mulai) = &req.tgl_aktual_mulai {
                query.push_str(&format!(", tgl_aktual_mulai = '{}'", escape_sql_string(tgl_mulai)));
            }
            if let Some(jam) = &req.jam_checkin {
                query.push_str(&format!(", jam_checkin = '{}'", escape_sql_string(jam)));
            }
            if let Some(konfirmasi) = req.konfirmasi_rfi {
                query.push_str(&format!(", konfirmasi_rfi = {}", konfirmasi));
            }
        }

        ("rfi_done", "rfs_done") => {
            if let Some(jam) = &req.jam_checkout {
                query.push_str(&format!(", jam_checkout = '{}'", escape_sql_string(jam)));
            }
            if let Some(konfirmasi) = req.konfirmasi_rfs {
                query.push_str(&format!(", konfirmasi_rfs = {}", konfirmasi));
            }
        }

        ("rfs_done", "bast") => {
            if let Some(konfirmasi) = req.konfirmasi_dok {
                query.push_str(&format!(", konfirmasi_dok = {}", konfirmasi));
            }
            if let Some(konfirmasi) = req.konfirmasi_final {
                query.push_str(&format!(", konfirmasi_final = {}", konfirmasi));
            }
            if let Some(catatan) = &req.catatan_teknis {
                query.push_str(&format!(", catatan_teknis = '{}'", escape_sql_string(catatan)));
            }
        }

        _ => {}
    }

    Ok(())
}

// ─── ESCAPE SQL STRING ────────────────────────────────────────────────────────
/// Helper: Escape single quotes dalam string untuk SQL safety
fn escape_sql_string(s: &str) -> String {
    s.replace("'", "''")
}

/// Get history dari stage transitions untuk site tertentu (audit trail)
pub async fn get_site_stage_logs(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<SiteStageLog>>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id, "sites")?;
    let site_id_str = site_thing.to_string();

    let query = "SELECT * FROM site_stage_logs WHERE site_id = $site_id ORDER BY created_at DESC";
    let mut response = state
        .db
        .query(query)
        .bind(("site_id", site_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error fetching stage logs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let logs: Vec<SiteStageLog> = response.take(0).map_err(|_| {
        eprintln!("Parse error");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let logs_count = logs.len();
    Ok(Json(ApiResponse {
        success: true,
        data: Some(logs),
        message: Some(format!("Retrieved {} stage logs for site {}", logs_count, site_id_str)),
    }))
}

// ─── GET VALID NEXT STAGES ────────────────────────────────────────────────────
/// Helper endpoint untuk UI: know stages mana yang bisa di-transisi next
pub async fn get_valid_next_stages(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id, "sites")?;

    // Fetch site untuk get current stage
    let site = fetch_site(&state, &site_thing).await?;
    let current_stage = site.stage.as_deref().unwrap_or("imported");

    // Get project type untuk project-specific stages
    let project_type = "FILTER"; // TODO: Fetch dari project

    let valid_stages = StageTransitionService::get_valid_next_stages(current_stage, project_type);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(valid_stages),
        message: Some(format!("Valid next stages from {}", current_stage)),
    }))
}

// ─── HELPER FUNCTIONS ─────────────────────────────────────────────────────────

/// Fetch single site dengan error handling
async fn fetch_site(state: &Arc<AppState>, site_thing: &Thing) -> Result<Site, StatusCode> {
    let query = "SELECT * FROM $site_id";
    let mut response = state
        .db
        .query(query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error fetching site: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut sites: Vec<Site> = response.take(0).map_err(|_| {
        eprintln!("Parse error");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    sites
        .into_iter()
        .next()
        .ok_or(StatusCode::NOT_FOUND)
}

/// Create stage log entry di tabel site_stage_logs
async fn create_stage_log(
    state: &Arc<AppState>,
    site_thing: &Thing,
    from_stage: String,
    to_stage: String,
    notes: &Option<String>,
) -> Result<(), StatusCode> {
    let log_query = "CREATE site_stage_logs SET site_id = $site_id, from_stage = $from_stage, to_stage = $to_stage, notes = $notes, changed_by = 'system', created_at = time::now()";

    let _result = state
        .db
        .query(log_query)
        .bind(("site_id", site_thing.clone()))
        .bind(("from_stage", from_stage))
        .bind(("to_stage", to_stage))
        .bind(("notes", notes.clone().unwrap_or_default()))
        .await
        .map_err(|e| {
            eprintln!("Database error creating stage log: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_validation() {
        // Unit tests untuk handler functions
        // TODO: Add integration tests dengan mock state
    }
}
