use crate::extractors::FormOrJson;
use axum::{
    extract::{Path, State, Multipart, Request, FromRequest},
    http::{StatusCode, header, HeaderMap, HeaderValue},
    response::Response,
    body::Body,
    Json,
};
use std::sync::Arc;
use surrealdb::sql::Thing;
use base64::Engine;
use crate::models::{
    ApiResponse, Site, Termin, TerminFile, TerminWithSiteInfo, TerminSiteInfo, 
    CreateTerminRequest, UpdateTerminRequest,
    CreateTerminFileRequest, SubmitTerminRequest, ReviewTerminRequest, ApproveTerminRequest, PayTerminRequest,
};
use crate::state::AppState;

// ==================== TERMIN HANDLERS ====================

// Helper function to strip table prefix if present
fn strip_table_prefix<'a>(id_str: &'a str, table: &str) -> &'a str {
    let prefix = format!("{}:", table);
    id_str.strip_prefix(&prefix).unwrap_or(id_str)
}

pub async fn create_termin(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    // Detect Content-Type: support both JSON and multipart/form-data
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let req: CreateTerminRequest;
    let mut dok_file_bytes: Option<Vec<u8>> = None;
    let mut dok_file_name: Option<String> = None;
    let mut dok_mime_type_raw: Option<String> = None;

    if content_type.starts_with("multipart/form-data") {
        let mut multipart = Multipart::from_request(request, &state)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let mut project_id_opt: Option<String> = None;
        let mut site_id_opt: Option<String> = None;
        let mut type_termin_opt: Option<String> = None;
        let mut tgl_terima_opt: Option<String> = None;
        let mut jumlah_opt: Option<i64> = None;
        let mut termin_ke_opt: Option<i32> = None;
        let mut percentage_opt: Option<i32> = None;
        let mut status_opt: Option<String> = None;
        let mut keterangan_opt: Option<String> = None;
        let mut submitted_by_opt: Option<String> = None;
        let mut nomor_rekening_opt: Option<String> = None;

        while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
            let name = field.name().unwrap_or("").to_string();
            match name.as_str() {
                "dokumen_pengajuan" => {
                    dok_file_name = field.file_name().map(|s| s.to_string());
                    dok_mime_type_raw = field.content_type().map(|s| s.to_string());
                    let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !bytes.is_empty() { dok_file_bytes = Some(bytes.to_vec()); }
                }
                "project_id" => { project_id_opt = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?); }
                "site_id" => { site_id_opt = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?); }
                "type_termin" => { type_termin_opt = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?); }
                "tgl_terima" => { let t = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?; if !t.is_empty() { tgl_terima_opt = Some(t); } }
                "jumlah" => { let t = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?; jumlah_opt = t.parse().ok(); }
                "termin_ke" => { let t = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?; termin_ke_opt = t.parse().ok(); }
                "percentage" => { let t = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?; percentage_opt = t.parse().ok(); }
                "status" => { let t = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?; if !t.is_empty() { status_opt = Some(t); } }
                "keterangan" => { let t = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?; if !t.is_empty() { keterangan_opt = Some(t); } }
                "submitted_by" => { let t = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?; if !t.is_empty() { submitted_by_opt = Some(t); } }
                "nomor_rekening_tujuan" => { let t = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?; if !t.is_empty() { nomor_rekening_opt = Some(t); } }
                _ => {}
            }
        }

        req = CreateTerminRequest {
            project_id: project_id_opt.ok_or(StatusCode::BAD_REQUEST)?,
            site_id: site_id_opt.ok_or(StatusCode::BAD_REQUEST)?,
            type_termin: type_termin_opt.ok_or(StatusCode::BAD_REQUEST)?,
            tgl_terima: tgl_terima_opt,
            jumlah: jumlah_opt.ok_or(StatusCode::BAD_REQUEST)?,
            termin_ke: termin_ke_opt.ok_or(StatusCode::BAD_REQUEST)?,
            percentage: percentage_opt.ok_or(StatusCode::BAD_REQUEST)?,
            status: status_opt,
            keterangan: keterangan_opt,
            submitted_by: submitted_by_opt,
            nomor_rekening_tujuan: nomor_rekening_opt,
        };
    } else {
        // JSON fallback
        let Json(json_req) = Json::<CreateTerminRequest>::from_request(request, &state)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        req = json_req;
    }

    // Parse and clean IDs
    let project_id_clean = strip_table_prefix(&req.project_id, "projects");
    let project_thing = Thing::try_from(("projects", project_id_clean))
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let site_id_clean = strip_table_prefix(&req.site_id, "sites");
    let site_thing = Thing::try_from(("sites", site_id_clean))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Step 1: Fetch the site to get maximal_budget for validation
    let fetch_site_query = "SELECT * FROM $site_id";
    let mut site_result = state.db.query(fetch_site_query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let site: Option<Site> = site_result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let site = site.ok_or(StatusCode::NOT_FOUND)?;
    
    // Step 2: Validate termin_ke range (1–6 to support T1, T2a, T2b, T2c, T3, T4 split model)
    // T1=1(30%), T2a=2(15%), T2b=3(25%), T2c=4(10%), T3=5(10%), T4=6(10%)
    if req.termin_ke < 1 || req.termin_ke > 6 {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some(format!(
                "Validation failed: termin_ke must be between 1-6. Got: {}",
                req.termin_ke
            )),
        }));
    }
    if req.percentage < 1 || req.percentage > 100 {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some(format!(
                "Validation failed: percentage harus antara 1-100. Got: {}",
                req.percentage
            )),
        }));
    }
    
    // Step 3: Validate previous termin dependency (termin can only be created if previous termin is approved)
    if req.termin_ke > 1 {
        let previous_termin_ke = req.termin_ke - 1;
        let check_previous_query = r#"
            SELECT * FROM termins 
            WHERE site_id = $site_id 
            AND termin_ke = $previous_termin_ke 
            ORDER BY created_at DESC 
            LIMIT 1
        "#;
        
        let mut previous_result = state.db.query(check_previous_query)
            .bind(("site_id", site_thing.clone()))
            .bind(("previous_termin_ke", previous_termin_ke))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let previous_termin: Option<Termin> = previous_result.take(0)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        match previous_termin {
            None => {
                return Ok(Json(ApiResponse {
                    success: false,
                    data: None,
                    message: Some(format!(
                        "Validation failed: Termin {} belum dibuat. Termin harus dibuat secara berurutan.",
                        previous_termin_ke
                    )),
                }));
            }
            Some(prev) => {
                if prev.status != "approved" && prev.status != "paid" {
                    return Ok(Json(ApiResponse {
                        success: false,
                        data: None,
                        message: Some(format!(
                            "Validation failed: Termin {} harus disetujui direktur (status: approved) terlebih dahulu sebelum mengajukan Termin {}. Status Termin {} saat ini: {}",
                            previous_termin_ke, req.termin_ke, previous_termin_ke, prev.status
                        )),
                    }));
                }
            }
        }
    }
    
    // Step 4: Validate 70% maximum payment limit
    // Get sum of all existing termins for this site
    let sum_query = r#"
        SELECT * FROM termins 
        WHERE site_id = $site_id
    "#;
    
    let mut sum_result = state.db.query(sum_query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let existing_termins: Vec<Termin> = sum_result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let current_total: i64 = existing_termins.iter().map(|t| t.jumlah).sum();
    let max_allowed = (site.maximal_budget * 70) / 100; // 70% dari maximal_budget
    let new_total = current_total + req.jumlah;
    
    if new_total > max_allowed {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some(format!(
                "Validation failed: Total pembayaran (Rp {}) melebihi batas maksimal 70% dari nilai site (Rp {}). Total saat ini: Rp {}, Termin baru: Rp {}, Sisa kuota: Rp {}",
                new_total, max_allowed, current_total, req.jumlah, max_allowed - current_total
            )),
        }));
    }
    
    // Step 5: Determine status and submit tracking based on submitted_by
    let (status, submitted_by, _submitted_at) = if let Some(submitter) = &req.submitted_by {
        ("pending_review".to_string(), Some(submitter.clone()), Some("time::now()"))
    } else {
        (req.status.clone().unwrap_or_else(|| "draft".to_string()), None, None)
    };

    // Step 6: Process uploaded document (from multipart, if any)
    let (dok_data_url, dok_filename, dok_mime, dok_size): (Option<String>, Option<String>, Option<String>, Option<i64>) =
        if let Some(bytes) = dok_file_bytes {
            let mime = dok_mime_type_raw.unwrap_or_else(|| "application/octet-stream".to_string());
            let size = bytes.len() as i64;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            let data_url = format!("data:{};base64,{}", mime, b64);
            (Some(data_url), dok_file_name, Some(mime), Some(size))
        } else {
            (None, None, None, None)
        };

    // Step 7: Create the termin
    let query = if submitted_by.is_some() {
        r#"
        CREATE termins SET 
            project_id = $project_id,
            site_id = $site_id,
            type_termin = $type_termin,
            tgl_terima = $tgl_terima,
            jumlah = $jumlah,
            termin_ke = $termin_ke,
            percentage = $percentage,
            status = $status,
            keterangan = $keterangan,
            nomor_rekening_tujuan = $nomor_rekening_tujuan,
            submitted_by = $submitted_by,
            submitted_at = time::now(),
            reviewed_by = NONE,
            reviewed_at = NONE,
            catatan_review = NONE,
            approved_by = NONE,
            approved_at = NONE,
            catatan_approval = NONE,
            paid_by = NONE,
            paid_at = NONE,
            jumlah_dibayar = NONE,
            referensi_pembayaran = NONE,
            catatan_pembayaran = NONE,
            bukti_pembayaran = NONE,
            dokumen_pengajuan = $dokumen_pengajuan,
            dokumen_pengajuan_filename = $dokumen_pengajuan_filename,
            dokumen_pengajuan_mime_type = $dokumen_pengajuan_mime_type,
            dokumen_pengajuan_size = $dokumen_pengajuan_size,
            created_at = time::now(),
            updated_at = time::now()
        "#
    } else {
        r#"
        CREATE termins SET 
            project_id = $project_id,
            site_id = $site_id,
            type_termin = $type_termin,
            tgl_terima = $tgl_terima,
            jumlah = $jumlah,
            termin_ke = $termin_ke,
            percentage = $percentage,
            status = $status,
            keterangan = $keterangan,
            nomor_rekening_tujuan = $nomor_rekening_tujuan,
            submitted_by = NONE,
            submitted_at = NONE,
            reviewed_by = NONE,
            reviewed_at = NONE,
            catatan_review = NONE,
            approved_by = NONE,
            approved_at = NONE,
            catatan_approval = NONE,
            paid_by = NONE,
            paid_at = NONE,
            jumlah_dibayar = NONE,
            referensi_pembayaran = NONE,
            catatan_pembayaran = NONE,
            bukti_pembayaran = NONE,
            dokumen_pengajuan = $dokumen_pengajuan,
            dokumen_pengajuan_filename = $dokumen_pengajuan_filename,
            dokumen_pengajuan_mime_type = $dokumen_pengajuan_mime_type,
            dokumen_pengajuan_size = $dokumen_pengajuan_size,
            created_at = time::now(), 
            updated_at = time::now()
        "#
    };

    let mut query_builder = state.db.query(query)
        .bind(("project_id", project_thing))
        .bind(("site_id", site_thing))
        .bind(("type_termin", req.type_termin.clone()))
        .bind(("tgl_terima", req.tgl_terima.clone()))
        .bind(("jumlah", req.jumlah))
        .bind(("termin_ke", req.termin_ke))
        .bind(("percentage", req.percentage))
        .bind(("status", status))
        .bind(("keterangan", req.keterangan.clone()))
        .bind(("nomor_rekening_tujuan", req.nomor_rekening_tujuan.clone()))
        .bind(("dokumen_pengajuan", dok_data_url))
        .bind(("dokumen_pengajuan_filename", dok_filename))
        .bind(("dokumen_pengajuan_mime_type", dok_mime))
        .bind(("dokumen_pengajuan_size", dok_size));
    
    if let Some(submitter) = submitted_by {
        query_builder = query_builder.bind(("submitted_by", submitter));
    }
    
    let mut result = query_builder
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match termin {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: Some("Termin created successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_termins(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<TerminWithSiteInfo>>>, StatusCode> {
    // Fetch all termins
    let query = "SELECT * FROM termins ORDER BY created_at DESC";

    let mut result = state.db.query(query)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termins: Vec<Termin> = result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Map to TerminWithSiteInfo
    let mut termins_with_site: Vec<TerminWithSiteInfo> = Vec::new();
    
    for termin in termins {
        // Fetch site details
        let site_name = if let Some(ref site_id) = termin.site_id {
            let site_query = "SELECT * FROM type::thing($site_id)";
            let mut site_result = state.db.query(site_query)
                .bind(("site_id", site_id.to_string()))
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let site: Option<Site> = site_result.take(0)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            site.map(|s| s.site_name).unwrap_or_default()
        } else {
            String::new()
        };
        
        // Fetch project details
        let project_name = if let Some(ref project_id) = termin.project_id {
            let project_query = "SELECT * FROM type::thing($project_id)";
            let mut project_result = state.db.query(project_query)
                .bind(("project_id", project_id.to_string()))
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let project: Option<crate::models::Project> = project_result.take(0)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            project.map(|p| p.name).unwrap_or_default()
        } else {
            String::new()
        };
        
        // Build TerminWithSiteInfo
        let termin_with_site = TerminWithSiteInfo {
            id: termin.id,
            project_id: termin.project_id,
            site_id: Some(TerminSiteInfo { site_name, project_name }),
            type_termin: termin.type_termin,
            tgl_terima: termin.tgl_terima,
            jumlah: termin.jumlah,
            termin_ke: termin.termin_ke,
            percentage: termin.percentage,
            status: termin.status,
            keterangan: termin.keterangan,
            submitted_by: termin.submitted_by,
            submitted_at: termin.submitted_at,
            reviewed_by: termin.reviewed_by,
            reviewed_at: termin.reviewed_at,
            catatan_review: termin.catatan_review,
            approved_by: termin.approved_by,
            approved_at: termin.approved_at,
            catatan_approval: termin.catatan_approval,
            paid_by: termin.paid_by,
            paid_at: termin.paid_at,
            jumlah_dibayar: termin.jumlah_dibayar,
            referensi_pembayaran: termin.referensi_pembayaran,
            catatan_pembayaran: termin.catatan_pembayaran,
            bukti_pembayaran: termin.bukti_pembayaran,
            bukti_pembayaran_filename: termin.bukti_pembayaran_filename,
            bukti_pembayaran_mime_type: termin.bukti_pembayaran_mime_type,
            bukti_pembayaran_size: termin.bukti_pembayaran_size,
            nomor_rekening_tujuan: termin.nomor_rekening_tujuan,
            created_at: termin.created_at,
            updated_at: termin.updated_at,
        };
        
        termins_with_site.push(termin_with_site);
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(termins_with_site),
        message: None,
    }))
}

pub async fn get_termins_by_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<TerminWithSiteInfo>>>, StatusCode> {
    // Fetch all termins for the project
    let query = "SELECT * FROM termins WHERE project_id = type::thing('projects', $id) ORDER BY created_at DESC";

    let mut result = state.db.query(query)
        .bind(("id", project_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termins: Vec<Termin> = result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Map to TerminWithSiteInfo
    let mut termins_with_site: Vec<TerminWithSiteInfo> = Vec::new();
    
    for termin in termins {
        // Fetch site details
        let site_name = if let Some(ref site_id) = termin.site_id {
            let site_query = "SELECT * FROM type::thing($site_id)";
            let mut site_result = state.db.query(site_query)
                .bind(("site_id", site_id.to_string()))
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let site: Option<Site> = site_result.take(0)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            site.map(|s| s.site_name).unwrap_or_default()
        } else {
            String::new()
        };
        
        // Fetch project details
        let project_name = if let Some(ref project_id) = termin.project_id {
            let project_query = "SELECT * FROM type::thing($project_id)";
            let mut project_result = state.db.query(project_query)
                .bind(("project_id", project_id.to_string()))
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let project: Option<crate::models::Project> = project_result.take(0)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            project.map(|p| p.name).unwrap_or_default()
        } else {
            String::new()
        };
        
        // Build TerminWithSiteInfo
        let termin_with_site = TerminWithSiteInfo {
            id: termin.id,
            project_id: termin.project_id,
            site_id: Some(TerminSiteInfo { site_name, project_name }),
            type_termin: termin.type_termin,
            tgl_terima: termin.tgl_terima,
            jumlah: termin.jumlah,
            termin_ke: termin.termin_ke,
            percentage: termin.percentage,
            status: termin.status,
            keterangan: termin.keterangan,
            submitted_by: termin.submitted_by,
            submitted_at: termin.submitted_at,
            reviewed_by: termin.reviewed_by,
            reviewed_at: termin.reviewed_at,
            catatan_review: termin.catatan_review,
            approved_by: termin.approved_by,
            approved_at: termin.approved_at,
            catatan_approval: termin.catatan_approval,
            paid_by: termin.paid_by,
            paid_at: termin.paid_at,
            jumlah_dibayar: termin.jumlah_dibayar,
            referensi_pembayaran: termin.referensi_pembayaran,
            catatan_pembayaran: termin.catatan_pembayaran,
            bukti_pembayaran: termin.bukti_pembayaran,
            bukti_pembayaran_filename: termin.bukti_pembayaran_filename,
            bukti_pembayaran_mime_type: termin.bukti_pembayaran_mime_type,
            bukti_pembayaran_size: termin.bukti_pembayaran_size,
            nomor_rekening_tujuan: termin.nomor_rekening_tujuan,
            created_at: termin.created_at,
            updated_at: termin.updated_at,
        };
        
        termins_with_site.push(termin_with_site);
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(termins_with_site),
        message: None,
    }))
}

pub async fn get_termins_by_site(
    State(state): State<Arc<AppState>>,
    Path(site_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<TerminWithSiteInfo>>>, StatusCode> {
    // Fetch all termins for the site
    let query = "SELECT * FROM termins WHERE site_id = type::thing('sites', $id) ORDER BY created_at DESC";

    let mut result = state.db.query(query)
        .bind(("id", site_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termins: Vec<Termin> = result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Map to TerminWithSiteInfo
    let mut termins_with_site: Vec<TerminWithSiteInfo> = Vec::new();
    
    for termin in termins {
        // Fetch site details
        let site_name = if let Some(ref site_id) = termin.site_id {
            let site_query = "SELECT * FROM type::thing($site_id)";
            let mut site_result = state.db.query(site_query)
                .bind(("site_id", site_id.to_string()))
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let site: Option<Site> = site_result.take(0)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            site.map(|s| s.site_name).unwrap_or_default()
        } else {
            String::new()
        };
        
        // Fetch project details
        let project_name = if let Some(ref project_id) = termin.project_id {
            let project_query = "SELECT * FROM type::thing($project_id)";
            let mut project_result = state.db.query(project_query)
                .bind(("project_id", project_id.to_string()))
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let project: Option<crate::models::Project> = project_result.take(0)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            project.map(|p| p.name).unwrap_or_default()
        } else {
            String::new()
        };
        
        // Build TerminWithSiteInfo
        let termin_with_site = TerminWithSiteInfo {
            id: termin.id,
            project_id: termin.project_id,
            site_id: Some(TerminSiteInfo { site_name, project_name }),
            type_termin: termin.type_termin,
            tgl_terima: termin.tgl_terima,
            jumlah: termin.jumlah,
            termin_ke: termin.termin_ke,
            percentage: termin.percentage,
            status: termin.status,
            keterangan: termin.keterangan,
            submitted_by: termin.submitted_by,
            submitted_at: termin.submitted_at,
            reviewed_by: termin.reviewed_by,
            reviewed_at: termin.reviewed_at,
            catatan_review: termin.catatan_review,
            approved_by: termin.approved_by,
            approved_at: termin.approved_at,
            catatan_approval: termin.catatan_approval,
            paid_by: termin.paid_by,
            paid_at: termin.paid_at,
            jumlah_dibayar: termin.jumlah_dibayar,
            referensi_pembayaran: termin.referensi_pembayaran,
            catatan_pembayaran: termin.catatan_pembayaran,
            bukti_pembayaran: termin.bukti_pembayaran,
            bukti_pembayaran_filename: termin.bukti_pembayaran_filename,
            bukti_pembayaran_mime_type: termin.bukti_pembayaran_mime_type,
            bukti_pembayaran_size: termin.bukti_pembayaran_size,
            nomor_rekening_tujuan: termin.nomor_rekening_tujuan,
            created_at: termin.created_at,
            updated_at: termin.updated_at,
        };
        
        termins_with_site.push(termin_with_site);
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(termins_with_site),
        message: None,
    }))
}

pub async fn get_termin_by_id(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = "SELECT * FROM $termin_id";

    let mut result = state.db.query(query)
        .bind(("termin_id", thing))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match termin {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: None,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_termin(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
    FormOrJson(req): FormOrJson<UpdateTerminRequest>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Check if termin exists and is in draft status
    let check_query = "SELECT * FROM $termin_id";
    let mut check_result = state.db.query(check_query)
        .bind(("termin_id", thing.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let existing: Option<Termin> = check_result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if existing.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let existing = existing.unwrap();
    if existing.status != "draft" {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("Only draft termins can be updated".to_string()),
        }));
    }

    // Build dynamic update query
    let mut set_clauses = vec!["updated_at = time::now()"];
    let mut bindings = Vec::new();

    if req.type_termin.is_some() {
        set_clauses.push("type_termin = $type_termin");
        bindings.push(("type_termin", req.type_termin.clone()));
    }
    if req.tgl_terima.is_some() {
        set_clauses.push("tgl_terima = $tgl_terima");
        bindings.push(("tgl_terima", req.tgl_terima.clone()));
    }
    if req.jumlah.is_some() {
        set_clauses.push("jumlah = $jumlah");
    }
    if req.keterangan.is_some() {
        set_clauses.push("keterangan = $keterangan");
        bindings.push(("keterangan", req.keterangan.clone()));
    }

    let set_clause = set_clauses.join(", ");
    let query = format!("UPDATE $termin_id SET {}", set_clause);

    let mut db_query = state.db.query(&query)
        .bind(("termin_id", thing));

    for (key, value) in bindings {
        db_query = db_query.bind((key, value));
    }
    if let Some(jumlah) = req.jumlah {
        db_query = db_query.bind(("jumlah", jumlah));
    }

    let mut result = db_query.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let updated: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match updated {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: Some("Termin updated successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn submit_termin(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
    FormOrJson(req): FormOrJson<SubmitTerminRequest>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = r#"
        UPDATE $termin_id SET 
            status = 'pending_review',
            submitted_by = $submitted_by,
            submitted_at = time::now(),
            updated_at = time::now()
    "#;

    let mut result = state.db.query(query)
        .bind(("termin_id", thing))
        .bind(("submitted_by", req.submitter_name.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match termin {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: Some("Termin submitted for review".to_string()),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn review_termin(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
    FormOrJson(req): FormOrJson<ReviewTerminRequest>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let new_status = if req.approve { "reviewed" } else { "draft" };

    let query = r#"
        UPDATE $termin_id SET 
            status = $status,
            reviewed_by = $reviewed_by,
            reviewed_at = time::now(),
            catatan_review = $catatan_review,
            updated_at = time::now()
    "#;

    let mut result = state.db.query(query)
        .bind(("termin_id", thing))
        .bind(("status", new_status))
        .bind(("reviewed_by", req.reviewer_name.clone()))
        .bind(("catatan_review", req.catatan_review.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let message = if req.approve {
        "Termin reviewed and approved by Field Head. Waiting for Director approval.".to_string()
    } else {
        "Termin rejected by Field Head. Returned to draft.".to_string()
    };

    match termin {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: Some(message),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn approve_termin(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
    FormOrJson(req): FormOrJson<ApproveTerminRequest>,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Check if user has permission to approve
    let allowed_roles = vec!["admin", "direktur"];
    if !allowed_roles.contains(&req.approver_role.to_lowercase().as_str()) {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("Hanya Admin atau Direktur yang dapat menyetujui termin (Only Admin or Director can approve)".to_string()),
        }));
    }

    let new_status = if req.approve { "approved" } else { "draft" };

    let query = r#"
        UPDATE $termin_id SET 
            status = $status,
            approved_by = $approved_by,
            approved_at = time::now(),
            catatan_approval = $catatan_approval,
            updated_at = time::now()
    "#;

    let mut result = state.db.query(query)
        .bind(("termin_id", thing))
        .bind(("status", new_status))
        .bind(("approved_by", req.approver_name.clone()))
        .bind(("catatan_approval", req.catatan_approval.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let message = if req.approve {
        "Termin approved by Director. Waiting for payment by Finance.".to_string()
    } else {
        "Termin rejected by Director. Returned to draft.".to_string()
    };

    match termin {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: Some(message),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn pay_termin(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
    request: Request,
) -> Result<Json<ApiResponse<Termin>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Deteksi Content-Type untuk menentukan format request
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let bukti_pembayaran: Option<String>;
    let bukti_pembayaran_filename: Option<String>;
    let bukti_pembayaran_mime_type: Option<String>;
    let bukti_pembayaran_size: Option<i64>;
    let payer_name: String;
    let jumlah_dibayar: i64;
    let referensi_pembayaran: String;
    let catatan_pembayaran: Option<String>;

    if content_type.starts_with("multipart/form-data") {
        // Handle multipart/form-data (untuk upload file)
        let mut multipart = Multipart::from_request(request, &state)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let mut payer_name_opt: Option<String> = None;
        let mut jumlah_dibayar_opt: Option<i64> = None;
        let mut referensi_pembayaran_opt: Option<String> = None;
        let mut catatan_pembayaran_opt: Option<String> = None;
        let mut file_data: Option<Vec<u8>> = None;
        let mut file_content_type: Option<String> = None;
        let mut file_name: Option<String> = None;

        while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
            let name = field.name().unwrap_or("").to_string();
            
            match name.as_str() {
                "bukti_pembayaran" => {
                    file_name = field.file_name().map(|s| s.to_string());
                    file_content_type = field.content_type().map(|s| s.to_string());
                    let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    file_data = Some(bytes.to_vec());
                }
                "payer_name" | "approved_by" => {
                    let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    payer_name_opt = Some(text);
                }
                "jumlah_dibayar" => {
                    let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    jumlah_dibayar_opt = text.parse().ok();
                }
                "referensi_pembayaran" => {
                    let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    referensi_pembayaran_opt = Some(text);
                }
                "catatan_pembayaran" => {
                    let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    catatan_pembayaran_opt = Some(text);
                }
                _ => {}
            }
        }

        // Validasi field required
        payer_name = payer_name_opt.ok_or(StatusCode::BAD_REQUEST)?;
        jumlah_dibayar = jumlah_dibayar_opt.ok_or(StatusCode::BAD_REQUEST)?;
        referensi_pembayaran = referensi_pembayaran_opt.ok_or(StatusCode::BAD_REQUEST)?;
        catatan_pembayaran = catatan_pembayaran_opt;

        // Convert file ke base64 data URL dan simpan ke database
        if let Some(data) = file_data {
            let file_size = data.len() as i64;
            let mime_type = file_content_type.unwrap_or_else(|| "application/pdf".to_string());
            let base64_data = base64::engine::general_purpose::STANDARD.encode(&data);
            
            bukti_pembayaran = Some(format!("data:{};base64,{}", mime_type, base64_data));
            bukti_pembayaran_filename = file_name;
            bukti_pembayaran_mime_type = Some(mime_type);
            bukti_pembayaran_size = Some(file_size);
        } else {
            bukti_pembayaran = None;
            bukti_pembayaran_filename = None;
            bukti_pembayaran_mime_type = None;
            bukti_pembayaran_size = None;
        }
    } else {
        // Handle application/json (untuk pembayaran tanpa file)
        let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        
        let req: PayTerminRequest = serde_json::from_slice(&body_bytes)
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        payer_name = req.payer_name;
        jumlah_dibayar = req.jumlah_dibayar;
        referensi_pembayaran = req.referensi_pembayaran;
        catatan_pembayaran = req.catatan_pembayaran;
        bukti_pembayaran = req.bukti_pembayaran;
        bukti_pembayaran_filename = None;  // JSON mode doesn't include file upload
        bukti_pembayaran_mime_type = None;
        bukti_pembayaran_size = None;
    }

    // Update termin dengan data pembayaran
    let query = r#"
        UPDATE $termin_id SET 
            status = 'paid',
            paid_by = $paid_by,
            paid_at = time::now(),
            jumlah_dibayar = $jumlah_dibayar,
            referensi_pembayaran = $referensi_pembayaran,
            catatan_pembayaran = $catatan_pembayaran,
            bukti_pembayaran = $bukti_pembayaran,
            bukti_pembayaran_filename = $bukti_pembayaran_filename,
            bukti_pembayaran_mime_type = $bukti_pembayaran_mime_type,
            bukti_pembayaran_size = $bukti_pembayaran_size,
            updated_at = time::now()
    "#;

    let mut result = state.db.query(query)
        .bind(("termin_id", thing))
        .bind(("paid_by", payer_name))
        .bind(("jumlah_dibayar", jumlah_dibayar))
        .bind(("referensi_pembayaran", referensi_pembayaran))
        .bind(("catatan_pembayaran", catatan_pembayaran))
        .bind(("bukti_pembayaran", bukti_pembayaran))
        .bind(("bukti_pembayaran_filename", bukti_pembayaran_filename))
        .bind(("bukti_pembayaran_mime_type", bukti_pembayaran_mime_type))
        .bind(("bukti_pembayaran_size", bukti_pembayaran_size))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match termin {
        Some(termin) => Ok(Json(ApiResponse {
            success: true,
            data: Some(termin),
            message: Some("Payment confirmed. Termin completed.".to_string()),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn download_bukti_pembayaran(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Query termin untuk mendapatkan bukti pembayaran
    let query = "SELECT * FROM $termin_id";
    
    let mut result = state.db.query(query)
        .bind(("termin_id", thing))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match termin {
        Some(termin) => {
            // Validasi apakah ada bukti pembayaran
            let data_url = termin.bukti_pembayaran
                .ok_or(StatusCode::NOT_FOUND)?;
            
            let filename = termin.bukti_pembayaran_filename
                .unwrap_or_else(|| "bukti_pembayaran.pdf".to_string());
            let mime_type = termin.bukti_pembayaran_mime_type
                .unwrap_or_else(|| "application/pdf".to_string());

            // Parse data URL (format: data:mime/type;base64,...)
            let parts: Vec<&str> = data_url.split(',').collect();
            if parts.len() != 2 {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            let base64_data = parts[1];
            
            // Decode base64
            let file_bytes = base64::engine::general_purpose::STANDARD
                .decode(base64_data)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Buat response dengan headers untuk download
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&mime_type)
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
                    .unwrap_or(HeaderValue::from_static("attachment; filename=\"bukti_pembayaran.pdf\"")),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&file_bytes.len().to_string()).unwrap(),
            );

            let body = Body::from(file_bytes);
            let mut response = Response::new(body);
            *response.headers_mut() = headers;
            *response.status_mut() = StatusCode::OK;

            Ok(response)
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_termin(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Cascade delete: first delete associated files
    let delete_files = "DELETE termin_files WHERE termin_id = $termin_id";
    state.db.query(delete_files)
        .bind(("termin_id", thing.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Then delete the termin
    let delete_termin = "DELETE $termin_id";
    state.db.query(delete_termin)
        .bind(("termin_id", thing))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        message: Some("Termin and associated files deleted successfully".to_string()),
    }))
}

// ==================== TERMIN FILE HANDLERS ====================

pub async fn create_termin_file(
    State(state): State<Arc<AppState>>,
    FormOrJson(req): FormOrJson<CreateTerminFileRequest>,
) -> Result<Json<ApiResponse<TerminFile>>, StatusCode> {
    let query = r#"
        CREATE termin_files CONTENT {
            termin_id: type::thing($termin_id),
            category: $category,
            title: $title,
            filename: $filename,
            original_name: $original_name,
            bucket: $bucket,
            key: $key,
            mime_type: $mime_type,
            size: $size,
            disk: $disk,
            visibility: $visibility,
            uploaded_at: time::now(),
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let mut result = state.db.query(query)
        .bind(("termin_id", req.termin_id.clone()))
        .bind(("category", req.category.clone()))
        .bind(("title", req.title.clone()))
        .bind(("filename", req.filename.clone()))
        .bind(("original_name", req.original_name.clone()))
        .bind(("bucket", req.bucket.clone()))
        .bind(("key", req.key.clone()))
        .bind(("mime_type", req.mime_type.clone()))
        .bind(("size", req.size))
        .bind(("disk", req.disk.clone()))
        .bind(("visibility", req.visibility.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file: Option<TerminFile> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match file {
        Some(file) => Ok(Json(ApiResponse {
            success: true,
            data: Some(file),
            message: Some("Termin file uploaded successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_termin_files(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<TerminFile>>>, StatusCode> {
    let query = "SELECT * FROM termin_files WHERE termin_id = type::thing('termins', $id) ORDER BY uploaded_at DESC";

    let mut result = state.db.query(query)
        .bind(("id", termin_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let files: Vec<TerminFile> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(files),
        message: None,
    }))
}

pub async fn delete_termin_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let query = "DELETE type::thing('termin_files', $id)";

    let _result = state.db.query(query)
        .bind(("id", file_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some("Termin file deleted successfully".to_string()),
        message: None,
    }))
}

// ==================== MULTIPART FILE UPLOAD FOR TERMIN ====================

pub async fn upload_termin_file_multipart(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<TerminFile>>, StatusCode> {
    let mut title: Option<String> = None;
    let mut category: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_content_type: Option<String> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                file_content_type = field.content_type().map(|s| s.to_string());
                let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                file_data = Some(bytes.to_vec());
            }
            "title" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                title = Some(text);
            }
            "category" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                category = Some(text);
            }
            _ => {}
        }
    }

    // Validate required fields
    let file_bytes = file_data.ok_or(StatusCode::BAD_REQUEST)?;
    let filename = file_name.ok_or(StatusCode::BAD_REQUEST)?;
    let title_str = title.unwrap_or_else(|| filename.clone());
    let mime_type = file_content_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let file_size = file_bytes.len() as i64;

    // Convert to base64 data URL
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_bytes);
    let data_url = format!("data:{};base64,{}", mime_type, base64_data);

    // Save to database
    let query = r#"
        CREATE termin_files CONTENT {
            termin_id: type::thing('termins', $termin_id),
            category: $category,
            title: $title,
            filename: $filename,
            original_name: $filename,
            file_data: $file_data,
            key: $filename,
            mime_type: $mime_type,
            size: $size,
            uploaded_at: time::now(),
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let mut result = state.db.query(query)
        .bind(("termin_id", termin_id))
        .bind(("category", category))
        .bind(("title", title_str))
        .bind(("filename", filename))
        .bind(("file_data", data_url))
        .bind(("mime_type", mime_type))
        .bind(("size", file_size))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file: Option<TerminFile> = result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match file {
        Some(file) => Ok(Json(ApiResponse {
            success: true,
            data: Some(file),
            message: Some("Termin file uploaded successfully".to_string()),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn download_termin_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    let thing = Thing::try_from(("termin_files", file_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = "SELECT * FROM $file_id";
    let mut result = state.db.query(query)
        .bind(("file_id", thing))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file: Option<TerminFile> = result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match file {
        Some(file) => {
            let data_url = file.file_data.ok_or(StatusCode::NOT_FOUND)?;
            let filename = file.filename;
            let mime_type = file.mime_type;

            // Parse data URL
            let parts: Vec<&str> = data_url.split(',').collect();
            if parts.len() != 2 {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            let base64_data = parts[1];
            let file_bytes = base64::engine::general_purpose::STANDARD
                .decode(base64_data)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Build response with headers
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&mime_type)
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
                    .unwrap_or(HeaderValue::from_static("attachment")),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&file_bytes.len().to_string()).unwrap(),
            );

            let body = Body::from(file_bytes);
            let mut response = Response::new(body);
            *response.headers_mut() = headers;
            *response.status_mut() = StatusCode::OK;

            Ok(response)
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_termin_director_summary(
    State(state): State<Arc<AppState>>,
    Path(termin_id): Path<String>,
) -> Result<Json<ApiResponse<crate::models::TerminDirectorSummaryResponse>>, StatusCode> {
    use crate::models::{Termin, Site, Project, Material};
    use surrealdb::sql::Thing;

    let thing = Thing::try_from(("termins", termin_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Fetch Termin
    let query_termin = "SELECT * FROM type::thing($id)";
    let mut termin_result = state.db.query(query_termin)
        .bind(("id", thing.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let termin: Option<Termin> = termin_result.take(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let termin = termin.ok_or(StatusCode::NOT_FOUND)?;

    let termin_ke_value = termin.termin_ke.unwrap_or(0);
    // Config needed here... use a default logic if parsing fails
    let required_stage = if termin_ke_value == 1 {
        "permit_ready"
    } else if termin_ke_value == 2 || termin.type_termin == "T2a" {
        "akses_ready"
    } else if termin.type_termin == "T2b" {
        "implementasi"
    } else if termin.type_termin == "T2c" {
        "rfi_done"
    } else if termin_ke_value == 3 || termin.type_termin == "T3" {
        "bast"
    } else if termin_ke_value == 4 || termin.type_termin == "T4" {
        "invoice"
    } else {
        ""
    };

    let mut current_stage = String::from("imported");
    let mut site_name_str = None;
    let mut project_name_str = None;
    let mut is_compliant = true;

    // Fetch Site Data
    if let Some(ref site_thing) = termin.site_id {
        let mut site_result = state.db.query("SELECT * FROM type::thing($site_id)")
            .bind(("site_id", site_thing.clone()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(site) = site_result.take::<Option<Site>>(0).unwrap_or(None) {
            current_stage = site.stage.clone().unwrap_or_else(|| "imported".to_string());
            site_name_str = Some(site.site_name.clone());

            // A simple placeholder stage hierarchy validation wrapper
            let allowed_stages = vec![
                "imported", "assigned", "survey", "survey_nok", "erfin_process", "erfin_ready",
                "permit_process", "permit_ready", "akses_process", "akses_ready", "implementasi",
                "rfi_done", "rfs_done", "dokumen_done", "bast", "invoice", "completed"
            ];
            
            let cur_idx = allowed_stages.iter().position(|&r| r == current_stage).unwrap_or(0);
            let req_idx = allowed_stages.iter().position(|&r| r == required_stage).unwrap_or(0);
            if req_idx > cur_idx {
                is_compliant = false;
            }
        }
    }

    // Fetch Project Data
    if let Some(ref proj_thing) = termin.project_id {
        let mut proj_result = state.db.query("SELECT * FROM type::thing($proj_id)")
            .bind(("proj_id", proj_thing.clone()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(proj) = proj_result.take::<Option<Project>>(0).unwrap_or(None) {
            project_name_str = Some(proj.name.clone());
        }
    }

    // Fetch Materials
    let mut materials: Vec<Material> = Vec::new();
    let mut total_material_items: i64 = 0;

    if let Some(ref site_thing) = termin.site_id {
        let mat_query = "SELECT * FROM materials WHERE site_id = type::thing($site_id) ORDER BY created_at DESC";
        if let Ok(mut mat_result) = state.db.query(mat_query).bind(("site_id", site_thing.clone())).await {
            materials = mat_result.take(0).unwrap_or_default();
            for m in &materials {
                total_material_items += m.qty;
            }
        }
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(crate::models::TerminDirectorSummaryResponse {
            termin,
            site_name: site_name_str,
            project_name: project_name_str,
            current_stage,
            required_stage: required_stage.to_string(),
            is_stage_compliant: is_compliant,
            total_material_items,
            materials,
        }),
        message: Some("Termin director summary generated".to_string()),
    }))
}
