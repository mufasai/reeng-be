use axum::{
    extract::{FromRequest, Json, Multipart, Request},
    http::{header, StatusCode},
};
use base64::Engine;
use chrono::{DateTime, NaiveDate, Utc};
use std::sync::Arc;
use surrealdb::sql::Thing;

use crate::models::{
    ApiResponse, CreateSiteRequest, UpdateSiteRequest, UpdateSiteStageRequest,
    Site, SiteStageLog, Team, SiteTeamMember, SiteTeamMemberDetail,
    AddSiteTeamMemberRequest, UpdateSiteTeamMemberRequest, TeamMasterInfo,
    SiteBoq, CreateSiteBoqRequest, UpdateSiteBoqRequest,
    Skp, CreateSkpRequest, UpdateSkpRequest,
    SitePermitDoc,
    SiteEvidence,
    SiteIssue, CreateSiteIssueRequest, ResolveSiteIssueRequest,
};
use crate::state::AppState;

// Helper function to strip table prefix from ID strings
fn strip_table_prefix<'a>(id_str: &'a str, table: &str) -> &'a str {
    let prefix = format!("{}:", table);
    id_str.strip_prefix(&prefix).unwrap_or(id_str)
}

pub async fn create_site(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<CreateSiteRequest>,
) -> Result<Json<ApiResponse<Site>>, StatusCode> {
    // Parse and clean project_id
    let project_id_clean = strip_table_prefix(&req.project_id, "projects");
    let project_thing = Thing::try_from(("projects", project_id_clean))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Build query to create site with proper record references
    let query = "CREATE sites SET project_id = $project_id, site_name = $site_name, site_info = $site_info, pekerjaan = $pekerjaan, lokasi = $lokasi, latitude = $latitude, longitude = $longitude, nomor_kontrak = $nomor_kontrak, start = $start, end = $end, maximal_budget = $maximal_budget, cost_estimated = $cost_estimated, pemberi_tugas = $pemberi_tugas, penerima_tugas = $penerima_tugas, site_document = $site_document, created_at = time::now(), updated_at = time::now()";

    let mut response = state
        .db
        .query(query)
        .bind(("project_id", project_thing.clone()))
        .bind(("site_name", req.site_name.clone()))
        .bind(("site_info", req.site_info.clone()))
        .bind(("pekerjaan", req.pekerjaan.clone()))
        .bind(("lokasi", req.lokasi.clone()))
        .bind(("latitude", req.latitude.clone()))
        .bind(("longitude", req.longitude.clone()))
        .bind(("nomor_kontrak", req.nomor_kontrak.clone()))
        .bind(("start", req.start.clone()))
        .bind(("end", req.end.clone()))
        .bind(("maximal_budget", req.maximal_budget))
        .bind(("cost_estimated", req.cost_estimated))
        .bind(("pemberi_tugas", req.pemberi_tugas.clone()))
        .bind(("penerima_tugas", req.penerima_tugas.clone()))
        .bind(("site_document", req.site_document.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error creating site: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut sites: Vec<Site> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    enrich_sites_timing_fields(&mut sites);

    let created_site = sites.into_iter().next().ok_or_else(|| {
        eprintln!("No site returned after creation");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // If team members are provided, create a team for this site
    if let Some(member_ids) = req.team_members {
        if !member_ids.is_empty() {
            let site_id_str = created_site.id.as_ref()
                .map(|t| t.to_string())
.ok_or_else(|| {
                    eprintln!("Site ID not found after creation");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            // Create team using query
            let site_id_clean = strip_table_prefix(&site_id_str, "sites");
            let site_thing = Thing::try_from(("sites", site_id_clean))
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            let team_query = "CREATE teams SET nama = $nama, project_id = $project_id, site_id = $site_id, active = $active, created_at = time::now(), updated_at = time::now()";

            let mut team_response = state
                .db
                .query(team_query)
                .bind(("nama", format!("Team {}", req.site_name)))
                .bind(("project_id", project_thing.clone()))
                .bind(("site_id", site_thing))
                .bind(("active", true))
                .await
                .map_err(|e| {
                    eprintln!("Database error creating team: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            let teams: Vec<Team> = team_response.take(0).map_err(|e| {
                eprintln!("Parse error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            if let Some(team) = teams.into_iter().next() {
                let team_id_str = team.id.as_ref()
                    .map(|t| t.to_string())
                    .unwrap_or_default();

                // Add team members
                let team_id_clean = strip_table_prefix(&team_id_str, "teams");
                let team_thing = Thing::try_from(("teams", team_id_clean))
                    .map_err(|_| StatusCode::BAD_REQUEST)?;

                for people_id_str in member_ids {
                    let people_id_clean = strip_table_prefix(&people_id_str, "people");
                    let people_thing = Thing::try_from(("people", people_id_clean))
                        .map_err(|_| StatusCode::BAD_REQUEST)?;

                    let member_query = "CREATE team_peoples SET team_id = $team_id, people_id = $people_id, created_at = time::now(), updated_at = time::now()";

                    let _ = state
                        .db
                        .query(member_query)
                        .bind(("team_id", team_thing.clone()))
                        .bind(("people_id", people_thing))
                        .await
                        .map_err(|e| {
                            eprintln!("Database error creating team_people: {}", e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })?;
                }
            }
        }
    }

    let mut created_site = created_site;
    enrich_site_timing_fields(&mut created_site);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(created_site),
        message: Some("Site created successfully".to_string()),
    }))
}

pub async fn list_sites(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Site>>>, StatusCode> {
    let mut response = state
        .db
        .query("SELECT * FROM sites ORDER BY created_at DESC")
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut sites: Vec<Site> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    enrich_sites_timing_fields(&mut sites);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(sites),
        message: None,
    }))
}

pub async fn get_sites_by_project(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Vec<Site>>>, StatusCode> {
    let project_thing = parse_thing_id(&project_id)?;

    let mut response = state
        .db
        .query("SELECT * FROM sites WHERE project_id = $project_id ORDER BY created_at DESC")
        .bind(("project_id", project_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let sites: Vec<Site> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(sites),
        message: None,
    }))
}

/// GET /api/sites/:id
pub async fn get_site_by_id(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Site>>, StatusCode> {
    let site_thing = Thing::try_from(site_id.as_str()).map_err(|_| {
        // coba prefix bila tidak ada kolon
        StatusCode::BAD_REQUEST
    })?;

    let mut response = state
        .db
        .query("SELECT * FROM type::thing($site_id)")
        .bind(("site_id", site_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error getting site by id: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut sites: Vec<Site> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    enrich_sites_timing_fields(&mut sites);

    if sites.is_empty() {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("Site tidak ditemukan".to_string()),
        }));
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(sites.remove(0)),
        message: None,
    }))
}

/// GET /api/sites/type?type={value}
/// Filter sites by type (case-insensitive search on site_info and pekerjaan fields)
pub async fn list_sites_by_type(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<Site>>>, StatusCode> {
    let type_filter = params.get("type")
        .ok_or_else(|| {
            eprintln!("Missing 'type' query parameter");
            StatusCode::BAD_REQUEST
        })?
        .to_lowercase();

    if type_filter.is_empty() {
        return Ok(Json(ApiResponse {
            success: true,
            data: Some(vec![]),
            message: Some("Type filter is empty".to_string()),
        }));
    }

    let mut response = state
        .db
        .query("SELECT * FROM sites ORDER BY created_at DESC")
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut sites: Vec<Site> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Client-side filtering (case-insensitive)
    sites.retain(|site| {
        let site_info = site.site_info.as_str().to_lowercase();
        let pekerjaan = site.pekerjaan.as_str().to_lowercase();
        site_info.contains(&type_filter) || pekerjaan.contains(&type_filter)
    });

    enrich_sites_timing_fields(&mut sites);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(sites),
        message: None,
    }))
}

/// GET /api/sites/category/{category}
/// Filter sites by category (BLACKSITE, COMBAT, FILTER, L2H, REFINEN)
pub async fn list_sites_by_category(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(category): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Vec<Site>>>, StatusCode> {
    // Normalize category name to standard format
    let standard_category = match category.to_lowercase().as_str() {
        "blacksite" | "black" | "bs" => "BLACK SITE",
        "combat" | "cb" => "COMBAT",
        "filter" | "ft" | "filter_site" => "FILTER",
        "l2h" | "l2h_site" => "L2H",
        "refinen" | "refinery" | "ref" => "REFINEN",
        _ => return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some(format!("Invalid category: {}. Supported: blacksite, combat, filter, l2h, refinen", category)),
        })),
    };

    let mut response = state
        .db
        .query("SELECT * FROM sites ORDER BY created_at DESC")
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut sites: Vec<Site> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Filter by category (search in site_info and pekerjaan fields)
    sites.retain(|site| {
        let site_info = site.site_info.as_str().to_uppercase();
        let pekerjaan = site.pekerjaan.as_str().to_uppercase();
        site_info.contains(standard_category) || pekerjaan.contains(standard_category)
    });

    enrich_sites_timing_fields(&mut sites);

    if sites.is_empty() {
        return Ok(Json(ApiResponse {
            success: true,
            data: Some(sites),
            message: Some(format!("No sites found for category: {}", standard_category)),
        }));
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(sites),
        message: None,
    }))
}

// Helper function to parse "table:id" string into Thing
fn parse_thing_id(id_str: &str) -> Result<Thing, StatusCode> {
    // Use from_string to parse Thing from "table:id" format
    Thing::try_from(id_str).map_err(|_| {
        eprintln!("Failed to parse Thing from '{}'", id_str);
        StatusCode::BAD_REQUEST
    })
}

fn parse_bool_loose(value: &str) -> Option<bool> {
    match value.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Some(true),
        "false" | "0" | "no" | "n" => Some(false),
        _ => None,
    }
}

fn parse_datetime_to_utc(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn parse_date_to_utc_start(value: &str) -> Option<DateTime<Utc>> {
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return date
            .and_hms_opt(0, 0, 0)
            .map(|naive| DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
    }

    parse_datetime_to_utc(value)
}

fn non_negative_days_between(start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
    let diff = end.signed_duration_since(start).num_days();
    if diff < 0 { 0 } else { diff }
}

fn enrich_site_timing_fields(site: &mut Site) {
    let now = Utc::now();

    if let Some(stage_updated_at) = site.stage_updated_at.as_deref() {
        if let Some(stage_started_at) = parse_datetime_to_utc(stage_updated_at)
            .or_else(|| parse_date_to_utc_start(stage_updated_at))
        {
            site.days_in_stage = Some(non_negative_days_between(stage_started_at, now));
        }
    }

    let permit_start = site
        .tgl_berlaku_permit_tpas
        .as_deref()
        .and_then(parse_date_to_utc_start)
        .or_else(|| site.permit_date.as_deref().and_then(parse_date_to_utc_start));

    let permit_end = site
        .tgl_berakhir_permit_tpas
        .as_deref()
        .and_then(parse_date_to_utc_start);

    if let (Some(start), Some(end)) = (permit_start, permit_end) {
        site.permit_days_total = Some(non_negative_days_between(start, end));
        site.permit_days_elapsed = Some(non_negative_days_between(start, now));

        let remaining = end.signed_duration_since(now).num_days();
        site.permit_days_remaining = Some(if remaining < 0 { 0 } else { remaining });
    }
}

fn enrich_sites_timing_fields(sites: &mut [Site]) {
    for site in sites {
        enrich_site_timing_fields(site);
    }
}

pub async fn update_site(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
    Json(req): Json<UpdateSiteRequest>,
) -> Result<Json<ApiResponse<Site>>, StatusCode> {
    // Parse site_id
    let site_thing = parse_thing_id(&site_id)?;

    // First, check if the site exists
    let check_query = "SELECT * FROM type::thing($site_id)";
    let mut check_response = state
        .db
        .query(check_query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error checking site: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let existing_sites: Vec<Site> = check_response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if existing_sites.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Build update query dynamically based on provided fields
    let mut update_parts = vec!["updated_at = time::now()".to_string()];
    
    if req.project_id.is_some() {
        update_parts.push("project_id = $project_id".to_string());
    }
    if req.site_name.is_some() {
        update_parts.push("site_name = $site_name".to_string());
    }
    if req.site_info.is_some() {
        update_parts.push("site_info = $site_info".to_string());
    }
    if req.pekerjaan.is_some() {
        update_parts.push("pekerjaan = $pekerjaan".to_string());
    }
    if req.lokasi.is_some() {
        update_parts.push("lokasi = $lokasi".to_string());
    }
    if req.latitude.is_some() {
        update_parts.push("latitude = $latitude".to_string());
    }
    if req.longitude.is_some() {
        update_parts.push("longitude = $longitude".to_string());
    }
    if req.nomor_kontrak.is_some() {
        update_parts.push("nomor_kontrak = $nomor_kontrak".to_string());
    }
    if req.start.is_some() {
        update_parts.push("start = $start".to_string());
    }
    if req.end.is_some() {
        update_parts.push("end = $end".to_string());
    }
    if req.maximal_budget.is_some() {
        update_parts.push("maximal_budget = $maximal_budget".to_string());
    }
    if req.cost_estimated.is_some() {
        update_parts.push("cost_estimated = $cost_estimated".to_string());
    }
    if req.pemberi_tugas.is_some() {
        update_parts.push("pemberi_tugas = $pemberi_tugas".to_string());
    }
    if req.penerima_tugas.is_some() {
        update_parts.push("penerima_tugas = $penerima_tugas".to_string());
    }
    if req.site_document.is_some() {
        update_parts.push("site_document = $site_document".to_string());
    }

    let update_query = format!(
        "UPDATE type::thing($site_id) SET {}",
        update_parts.join(", ")
    );

    let mut query_builder = state.db.query(&update_query).bind(("site_id", site_thing));

    // Bind all the optional parameters
    if let Some(project_id) = req.project_id {
        let project_id_clean = strip_table_prefix(&project_id, "projects");
        let project_thing = Thing::try_from(("projects", project_id_clean))
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        query_builder = query_builder.bind(("project_id", project_thing));
    }
    if let Some(site_name) = req.site_name {
        query_builder = query_builder.bind(("site_name", site_name));
    }
    if let Some(site_info) = req.site_info {
        query_builder = query_builder.bind(("site_info", site_info));
    }
    if let Some(pekerjaan) = req.pekerjaan {
        query_builder = query_builder.bind(("pekerjaan", pekerjaan));
    }
    if let Some(lokasi) = req.lokasi {
        query_builder = query_builder.bind(("lokasi", lokasi));
    }
    if let Some(latitude) = req.latitude {
        query_builder = query_builder.bind(("latitude", latitude));
    }
    if let Some(longitude) = req.longitude {
        query_builder = query_builder.bind(("longitude", longitude));
    }
    if let Some(nomor_kontrak) = req.nomor_kontrak {
        query_builder = query_builder.bind(("nomor_kontrak", nomor_kontrak));
    }
    if let Some(start) = req.start {
        query_builder = query_builder.bind(("start", start));
    }
    if let Some(end) = req.end {
        query_builder = query_builder.bind(("end", end));
    }
    if let Some(maximal_budget) = req.maximal_budget {
        query_builder = query_builder.bind(("maximal_budget", maximal_budget));
    }
    if let Some(cost_estimated) = req.cost_estimated {
        query_builder = query_builder.bind(("cost_estimated", cost_estimated));
    }
    if let Some(pemberi_tugas) = req.pemberi_tugas {
        query_builder = query_builder.bind(("pemberi_tugas", pemberi_tugas));
    }
    if let Some(penerima_tugas) = req.penerima_tugas {
        query_builder = query_builder.bind(("penerima_tugas", penerima_tugas));
    }
    if let Some(site_document) = req.site_document {
        query_builder = query_builder.bind(("site_document", site_document));
    }

    let mut response = query_builder.await.map_err(|e| {
        eprintln!("Database error updating site: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let updated_sites: Vec<Site> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut updated_site = updated_sites.into_iter().next().ok_or_else(|| {
        eprintln!("No site returned after update");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    enrich_site_timing_fields(&mut updated_site);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(updated_site),
        message: Some("Site updated successfully".to_string()),
    }))
}

pub async fn delete_site(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // Parse site_id
    let site_thing = parse_thing_id(&site_id)?;

    // First, check if the site exists
    let check_query = "SELECT * FROM type::thing($site_id)";
    let mut check_response = state
        .db
        .query(check_query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error checking site: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let existing_sites: Vec<Site> = check_response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if existing_sites.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Delete related team_peoples records first
    let delete_team_peoples_query = "
        DELETE team_peoples WHERE team_id IN (
            SELECT id FROM teams WHERE site_id = type::thing($site_id)
        )
    ";
    
    state
        .db
        .query(delete_team_peoples_query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting team_peoples: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Delete teams associated with this site
    let delete_teams_query = "DELETE teams WHERE site_id = type::thing($site_id)";
    
    state
        .db
        .query(delete_teams_query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting teams: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Delete site_files associated with this site
    let delete_files_query = "DELETE site_files WHERE site_id = type::thing($site_id)";
    
    state
        .db
        .query(delete_files_query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting site_files: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Delete costs associated with this site
    let delete_costs_query = "DELETE costs WHERE site_id = type::thing($site_id)";
    
    state
        .db
        .query(delete_costs_query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting costs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Delete materials associated with this site
    let delete_materials_query = "DELETE materials WHERE site_id = type::thing($site_id)";
    
    state
        .db
        .query(delete_materials_query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting materials: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Delete termins associated with this site
    let delete_termins_query = "DELETE termins WHERE site_id = type::thing($site_id)";
    
    state
        .db
        .query(delete_termins_query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting termins: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Delete site_team_members (Tim Struktur) associated with this site
    let delete_site_team_members_query = "DELETE site_team_members WHERE site_id = $site_id";

    state
        .db
        .query(delete_site_team_members_query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting site_team_members: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Finally, delete the site itself
    let delete_query = "DELETE type::thing($site_id)";
    
    state
        .db
        .query(delete_query)
        .bind(("site_id", site_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting site: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("Site and all related data deleted successfully".to_string()),
    }))
}

// ==================== TIM STRUKTUR (SITE TEAM STRUCTURE) HANDLERS ====================

/// GET /api/sites/:site_id/team-structure
/// List all members in a site's Tim Struktur, enriched with master team data
pub async fn get_site_team_structure(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Vec<SiteTeamMemberDetail>>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id)?;

    let query = "SELECT * FROM site_team_members WHERE site_id = $site_id ORDER BY created_at ASC";

    let mut response = state
        .db
        .query(query)
        .bind(("site_id", site_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error getting site team structure: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let members: Vec<SiteTeamMember> = response.take(0).map_err(|e| {
        eprintln!("Parse error site_team_members: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Enrich each member entry with master team data (N queries, acceptable for small teams)
    let mut details: Vec<SiteTeamMemberDetail> = Vec::new();
    for member in members {
        let mut detail = SiteTeamMemberDetail {
            id: member.id,
            site_id: member.site_id,
            team_master_id: member.team_master_id.clone(),
            role: member.role,
            vendor: member.vendor,
            nik: None,
            nama: None,
            no_hp: None,
            jabatan: None,
            regional: None,
            created_at: member.created_at,
            updated_at: member.updated_at,
        };

        if let Some(ref team_thing) = member.team_master_id {
            let master_query = "SELECT nik, nama_karyawan, no_hp, jabatan_kerja, regional FROM type::thing($team_id)";
            if let Ok(mut master_res) = state
                .db
                .query(master_query)
                .bind(("team_id", team_thing.clone()))
                .await
            {
                let infos: Vec<TeamMasterInfo> = master_res.take(0).unwrap_or_default();
                if let Some(info) = infos.into_iter().next() {
                    detail.nik = info.nik;
                    detail.nama = info.nama_karyawan;
                    detail.no_hp = info.no_hp;
                    detail.jabatan = info.jabatan_kerja;
                    detail.regional = info.regional;
                }
            }
        }

        details.push(detail);
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(details),
        message: None,
    }))
}

/// POST /api/sites/:site_id/team-structure
/// Add a master team member to a site's Tim Struktur
pub async fn add_site_team_member(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
    Json(req): Json<AddSiteTeamMemberRequest>,
) -> Result<Json<ApiResponse<SiteTeamMemberDetail>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id)?;

    let team_master_id_clean = strip_table_prefix(&req.team_master_id, "teams");
    let team_master_thing = Thing::try_from(("teams", team_master_id_clean))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Prevent duplicate: same master team member in same site
    let check_query = "SELECT id FROM site_team_members WHERE site_id = $site_id AND team_master_id = $team_master_id LIMIT 1";
    let mut check_res = state
        .db
        .query(check_query)
        .bind(("site_id", site_thing.clone()))
        .bind(("team_master_id", team_master_thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error checking duplicate site team member: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let existing: Vec<SiteTeamMember> = check_res.take(0).unwrap_or_default();
    if !existing.is_empty() {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("Member already added to this site's team structure".to_string()),
        }));
    }

    // Insert into site_team_members
    let insert_query = "CREATE site_team_members SET \
        site_id = $site_id, \
        team_master_id = $team_master_id, \
        role = $role, \
        vendor = $vendor, \
        created_at = time::now(), \
        updated_at = time::now()";

    let mut insert_res = state
        .db
        .query(insert_query)
        .bind(("site_id", site_thing.clone()))
        .bind(("team_master_id", team_master_thing.clone()))
        .bind(("role", req.role.clone()))
        .bind(("vendor", req.vendor.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error creating site team member: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let created: Vec<SiteTeamMember> = insert_res.take(0).map_err(|e| {
        eprintln!("Parse error creating site team member: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let member = created.into_iter().next().ok_or_else(|| {
        eprintln!("No site_team_members record returned after creation");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Auto-update stage from "imported" to "assigned" when team member is added
    let _get_site_query = "SELECT stage FROM type::thing($site_id)";
    if let Ok(mut site_res) = state
        .db
        .query(_get_site_query)
        .bind(("site_id", site_thing.clone()))
        .await
    {
        let sites: Vec<Site> = site_res.take(0).unwrap_or_default();
        if let Some(site) = sites.first() {
            if let Some(current_stage) = &site.stage {
                if current_stage == "imported" {
                    let auto_update_query = "UPDATE type::thing($site_id) SET \
                        stage = 'assigned', \
                        stage_updated_at = time::now(), \
                        updated_at = time::now()";
                    
                    if let Err(e) = state
                        .db
                        .query(auto_update_query)
                        .bind(("site_id", site_thing.clone()))
                        .await
                    {
                        eprintln!("Warning: Failed to auto-update stage to assigned: {}", e);
                    } else {
                        eprintln!("✓ Stage auto-updated: imported → assigned");
                    }
                }
            }
        }
    }

    // Enrich with master team data
    let mut detail = SiteTeamMemberDetail {
        id: member.id,
        site_id: member.site_id,
        team_master_id: member.team_master_id,
        role: member.role,
        vendor: member.vendor,
        nik: None,
        nama: None,
        no_hp: None,
        jabatan: None,
        regional: None,
        created_at: member.created_at,
        updated_at: member.updated_at,
    };

    let master_query = "SELECT nik, nama_karyawan, no_hp, jabatan_kerja, regional FROM type::thing($team_id)";
    if let Ok(mut master_res) = state
        .db
        .query(master_query)
        .bind(("team_id", team_master_thing))
        .await
    {
        let infos: Vec<TeamMasterInfo> = master_res.take(0).unwrap_or_default();
        if let Some(info) = infos.into_iter().next() {
            detail.nik = info.nik;
            detail.nama = info.nama_karyawan;
            detail.no_hp = info.no_hp;
            detail.jabatan = info.jabatan_kerja;
            detail.regional = info.regional;
        }
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(detail),
        message: Some("Team member added to site structure successfully. Stage updated: imported → assigned".to_string()),
    }))
}

/// PUT /api/sites/:site_id/team-structure/:member_id
/// Update role/vendor of a member in a site's Tim Struktur
pub async fn update_site_team_member(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path((site_id, member_id)): axum::extract::Path<(String, String)>,
    Json(req): Json<UpdateSiteTeamMemberRequest>,
) -> Result<Json<ApiResponse<SiteTeamMemberDetail>>, StatusCode> {
    let _site_thing = parse_thing_id(&site_id)?;
    let member_thing = parse_thing_id(&member_id)?;

    // Build dynamic SET clause
    let mut set_parts = vec!["updated_at = time::now()".to_string()];
    if req.role.is_some() {
        set_parts.push("role = $role".to_string());
    }
    if req.vendor.is_some() {
        set_parts.push("vendor = $vendor".to_string());
    }

    let update_query = format!("UPDATE type::thing($member_id) SET {}", set_parts.join(", "));

    let mut qb = state
        .db
        .query(&update_query)
        .bind(("member_id", member_thing));

    if let Some(role) = req.role {
        qb = qb.bind(("role", role));
    }
    if let Some(vendor) = req.vendor {
        qb = qb.bind(("vendor", vendor));
    }

    let mut res = qb.await.map_err(|e| {
        eprintln!("Database error updating site team member: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let updated: Vec<SiteTeamMember> = res.take(0).map_err(|e| {
        eprintln!("Parse error updating site team member: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let member = updated.into_iter().next().ok_or(StatusCode::NOT_FOUND)?;

    // Enrich with master team data
    let mut detail = SiteTeamMemberDetail {
        id: member.id,
        site_id: member.site_id,
        team_master_id: member.team_master_id.clone(),
        role: member.role,
        vendor: member.vendor,
        nik: None,
        nama: None,
        no_hp: None,
        jabatan: None,
        regional: None,
        created_at: member.created_at,
        updated_at: member.updated_at,
    };

    if let Some(ref team_thing) = member.team_master_id {
        let master_query = "SELECT nik, nama_karyawan, no_hp, jabatan_kerja, regional FROM type::thing($team_id)";
        if let Ok(mut master_res) = state
            .db
            .query(master_query)
            .bind(("team_id", team_thing.clone()))
            .await
        {
            let infos: Vec<TeamMasterInfo> = master_res.take(0).unwrap_or_default();
            if let Some(info) = infos.into_iter().next() {
                detail.nik = info.nik;
                detail.nama = info.nama_karyawan;
                detail.no_hp = info.no_hp;
                detail.jabatan = info.jabatan_kerja;
                detail.regional = info.regional;
            }
        }
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(detail),
        message: Some("Tim Struktur member updated successfully".to_string()),
    }))
}

/// DELETE /api/sites/:site_id/team-structure/:member_id
/// Remove a member from a site's Tim Struktur
pub async fn remove_site_team_member(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path((site_id, member_id)): axum::extract::Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let _site_thing = parse_thing_id(&site_id)?;
    let member_thing = parse_thing_id(&member_id)?;

    let delete_query = "DELETE type::thing($member_id)";
    state
        .db
        .query(delete_query)
        .bind(("member_id", member_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error removing site team member: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("Team member removed from site structure".to_string()),
    }))
}

// ==================== STAGE HANDLERS ====================

/// POST /api/sites/:id/stage
/// Update stage site + catat log perubahan
/// Stage order: imported → assigned → permit_process → permit_ready →
///              akses_process → akses_ready → implementasi →
///              rfi_done → rfs_done → dokumen_done → bast → invoice → completed
pub async fn update_site_stage(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
    request: Request,
) -> Result<Json<ApiResponse<Site>>, StatusCode> {
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let req: UpdateSiteStageRequest;
    let mut permit_doc_file_bytes: Option<Vec<u8>> = None;
    let mut permit_doc_filename: Option<String> = None;
    let mut permit_doc_content_type: Option<String> = None;
    let mut permit_doc_type: Option<String> = None;
    let mut permit_doc_uploaded_by: Option<String> = None;
    let mut multiple_evidence_files: Vec<(String, String, Vec<u8>)> = Vec::new();

    if content_type.starts_with("multipart/form-data") {
        let mut multipart = Multipart::from_request(request, &state)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let mut stage_opt: Option<String> = None;
        let mut notes_opt: Option<String> = None;
        let mut changed_by_opt: Option<String> = None;
        let mut evidence_urls: Vec<String> = Vec::new();
        let mut permit_date_opt: Option<String> = None;
        let mut impl_cico_done_opt: Option<bool> = None;
        let mut impl_rfs_done_opt: Option<bool> = None;
        let mut impl_dokumen_done_opt: Option<bool> = None;
        let mut ineom_registered_opt: Option<bool> = None;
        let mut tpas_approved_opt: Option<bool> = None;
        let mut tp_approved_opt: Option<bool> = None;
        let mut caf_approved_opt: Option<bool> = None;
        let mut tgl_berlaku_permit_tpas_opt: Option<String> = None;
        let mut tgl_berakhir_permit_tpas_opt: Option<String> = None;
        let mut tower_provider_opt: Option<crate::models::TowerProvider> = None;
        let mut jenis_kunci_opt: Option<crate::models::JenisKunci> = None;
        let mut pic_akses_nama_opt: Option<String> = None;
        let mut pic_akses_telp_opt: Option<String> = None;
        let mut has_akses_gedung_opt: Option<bool> = None;
        let mut gedung_nama_opt: Option<String> = None;
        let mut gedung_pic_nama_opt: Option<String> = None;
        let mut gedung_pic_telp_opt: Option<String> = None;
        let mut gedung_akses_status_opt: Option<String> = None;
        let mut konfirmasi_akses_opt: Option<bool> = None;
        let mut tgl_rencana_implementasi_opt: Option<String> = None;
        let mut tgl_aktual_mulai_opt: Option<String> = None;
        let mut jam_checkin_opt: Option<String> = None;
        let mut jam_checkout_opt: Option<String> = None;
        let mut konfirmasi_rfi_opt: Option<bool> = None;
        let mut catatan_teknis_opt: Option<String> = None;
        
        // Survey & ERFIN options
        let mut survey_date_opt: Option<String> = None;
        let mut survey_result_opt: Option<String> = None;
        let mut survey_nok_reason_opt: Option<String> = None;
        let mut erfin_number_opt: Option<String> = None;
        let mut erfin_date_opt: Option<String> = None;
        let mut erfin_ready_date_opt: Option<String> = None;

        while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
            let name = field.name().unwrap_or("").to_string();
            match name.as_str() {
                "file" | "dokumen_tpas" | "dokumen_permit" | "permit_file" => {
                    let fn_opt = field.file_name().map(|s| s.to_string());
                    let ct_opt = field.content_type().map(|s| s.to_string());
                    let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !bytes.is_empty() {
                        permit_doc_filename = fn_opt;
                        permit_doc_content_type = ct_opt;
                        permit_doc_file_bytes = Some(bytes.to_vec());
                    }
                }
                "evidence_files" | "files" | "files[]" | "bukti_akses" => {
                    let fn_opt = field.file_name().unwrap_or("evidence.bin").to_string();
                    let ct_opt = field.content_type().unwrap_or("application/octet-stream").to_string();
                    let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !bytes.is_empty() {
                        multiple_evidence_files.push((fn_opt, ct_opt, bytes.to_vec()));
                    }
                }
                "doc_type" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        permit_doc_type = Some(value);
                    }
                }
                "uploaded_by" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        permit_doc_uploaded_by = Some(value);
                    }
                }
                "stage" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        stage_opt = Some(value);
                    }
                }
                "notes" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        notes_opt = Some(value);
                    }
                }
                "changed_by" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        changed_by_opt = Some(value);
                    }
                }
                "evidence_urls" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    let trimmed = value.trim();
                    if !trimmed.is_empty() {
                        evidence_urls.push(trimmed.to_string());
                    }
                }
                "permit_date" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        permit_date_opt = Some(value);
                    }
                }
                "impl_cico_done" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    impl_cico_done_opt = parse_bool_loose(&value);
                }
                "impl_rfs_done" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    impl_rfs_done_opt = parse_bool_loose(&value);
                }
                "impl_dokumen_done" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    impl_dokumen_done_opt = parse_bool_loose(&value);
                }
                "ineom_registered" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    ineom_registered_opt = parse_bool_loose(&value);
                }
                "tpas_approved" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    tpas_approved_opt = parse_bool_loose(&value);
                }
                "tp_approved" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    tp_approved_opt = parse_bool_loose(&value);
                }
                "caf_approved" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    caf_approved_opt = parse_bool_loose(&value);
                }
                "tgl_berlaku_permit_tpas" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        tgl_berlaku_permit_tpas_opt = Some(value);
                    }
                }
                "tgl_berakhir_permit_tpas" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        tgl_berakhir_permit_tpas_opt = Some(value);
                    }
                }
                "tower_provider" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        tower_provider_opt = match value.to_uppercase().as_str() {
                            "MITRATEL" => Some(crate::models::TowerProvider::Mitratel),
                            "STP" => Some(crate::models::TowerProvider::Stp),
                            "PTI" => Some(crate::models::TowerProvider::Pti),
                            "DMT" => Some(crate::models::TowerProvider::Dmt),
                            "LAINNYA" => Some(crate::models::TowerProvider::Lainnya),
                            _ => None,
                        };
                    }
                }
                "jenis_kunci" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        jenis_kunci_opt = match value.to_uppercase().as_str() {
                            "PADLOCK" => Some(crate::models::JenisKunci::Padlock),
                            "SMARTLOCK" => Some(crate::models::JenisKunci::Smartlock),
                            "QUADLOCK" => Some(crate::models::JenisKunci::Quadlock),
                            _ => None,
                        };
                    }
                }
                "pic_akses_nama" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        pic_akses_nama_opt = Some(value);
                    }
                }
                "pic_akses_telp" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        pic_akses_telp_opt = Some(value);
                    }
                }
                "has_akses_gedung" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    has_akses_gedung_opt = parse_bool_loose(&value);
                }
                "gedung_nama" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        gedung_nama_opt = Some(value);
                    }
                }
                "gedung_pic_nama" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        gedung_pic_nama_opt = Some(value);
                    }
                }
                "gedung_pic_telp" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        gedung_pic_telp_opt = Some(value);
                    }
                }
                "gedung_akses_status" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        gedung_akses_status_opt = Some(value);
                    }
                }
                "konfirmasi_akses" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    konfirmasi_akses_opt = parse_bool_loose(&value);
                }
                "tgl_rencana_implementasi" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        tgl_rencana_implementasi_opt = Some(value);
                    }
                }
                "tgl_aktual_mulai" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        tgl_aktual_mulai_opt = Some(value);
                    }
                }
                "jam_checkin" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        jam_checkin_opt = Some(value);
                    }
                }
                "jam_checkout" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        jam_checkout_opt = Some(value);
                    }
                }
                "konfirmasi_rfi" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    konfirmasi_rfi_opt = parse_bool_loose(&value);
                }
                "catatan_teknis" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() {
                        catatan_teknis_opt = Some(value);
                    }
                }
                "survey_date" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() { survey_date_opt = Some(value); }
                }
                "survey_result" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() { survey_result_opt = Some(value); }
                }
                "survey_nok_reason" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() { survey_nok_reason_opt = Some(value); }
                }
                "erfin_number" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() { erfin_number_opt = Some(value); }
                }
                "erfin_date" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() { erfin_date_opt = Some(value); }
                }
                "erfin_ready_date" => {
                    let value = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                    if !value.trim().is_empty() { erfin_ready_date_opt = Some(value); }
                }
                _ => {}
            }
        }

        req = UpdateSiteStageRequest {
            stage: stage_opt.ok_or(StatusCode::BAD_REQUEST)?,
            notes: notes_opt,
            changed_by: changed_by_opt,
            evidence_urls: if evidence_urls.is_empty() { None } else { Some(evidence_urls) },
            permit_date: permit_date_opt,
            impl_cico_done: impl_cico_done_opt,
            impl_rfs_done: impl_rfs_done_opt,
            impl_dokumen_done: impl_dokumen_done_opt,
            ineom_registered: ineom_registered_opt,
            tpas_approved: tpas_approved_opt,
            tp_approved: tp_approved_opt,
            caf_approved: caf_approved_opt,
            tgl_berlaku_permit_tpas: tgl_berlaku_permit_tpas_opt,
            tgl_berakhir_permit_tpas: tgl_berakhir_permit_tpas_opt,
            approval_chain: None,
            dokumen_tpas_url: None,
            tower_provider: tower_provider_opt,
            jenis_kunci: jenis_kunci_opt,
            pic_akses_nama: pic_akses_nama_opt,
            pic_akses_telp: pic_akses_telp_opt,
            survey_date: survey_date_opt,
            survey_result: survey_result_opt,
            survey_nok_reason: survey_nok_reason_opt,
            erfin_number: erfin_number_opt,
            erfin_date: erfin_date_opt,
            erfin_ready_date: erfin_ready_date_opt,
            has_akses_gedung: has_akses_gedung_opt,
            gedung_nama: gedung_nama_opt,
            gedung_pic_nama: gedung_pic_nama_opt,
            gedung_pic_telp: gedung_pic_telp_opt,
            gedung_akses_status: gedung_akses_status_opt,
            konfirmasi_akses: konfirmasi_akses_opt,
            tgl_rencana_implementasi: tgl_rencana_implementasi_opt,
            tgl_aktual_mulai: tgl_aktual_mulai_opt,
            jam_checkin: jam_checkin_opt,
            jam_checkout: jam_checkout_opt,
            konfirmasi_rfi: konfirmasi_rfi_opt,
            konfirmasi_rfs: None,
            konfirmasi_dok: None,
            konfirmasi_final: None,
            catatan_teknis: catatan_teknis_opt,
        };
    } else {
        let Json(json_req) = Json::<UpdateSiteStageRequest>::from_request(request, &state)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        req = json_req;
    }

    let valid_stages = [
        "imported", "assigned", "survey", "erfin_diproses", "erfin_ready", "permit_process", "permit_ready",
        "akses_process", "akses_ready", "implementasi",
        "rfi_done", "rfs_done", "dokumen_done", "bast", "invoice", "completed",
    ];
    if !valid_stages.contains(&req.stage.as_str()) {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some(format!("Stage '{}' tidak valid", req.stage)),
        }));
    }

    let site_thing = parse_thing_id(&site_id)?;

    // Ambil stage lama sebelum diupdate
    let get_query = "SELECT stage FROM type::thing($site_id)";
    let mut get_res = state
        .db
        .query(get_query)
        .bind(("site_id", site_thing.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error getting current stage: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    #[derive(serde::Deserialize)]
    struct StageOnly { stage: Option<String> }
    let rows: Vec<StageOnly> = get_res.take(0).unwrap_or_default();
    let from_stage = rows.into_iter().next()
        .and_then(|r| r.stage)
        .unwrap_or_else(|| "imported".to_string());

    // Validasi field wajib survey - erfin
    if req.stage == "survey" && (req.survey_date.is_none() || req.survey_date.as_deref() == Some("")) {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("survey_date (Tanggal Survey) wajib diisi saat masuk stage survey".to_string()),
        }));
    }
    if req.stage == "erfin_diproses" && (req.survey_result.is_none() || req.survey_result.as_deref() == Some("")) {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("survey_result (Hasil Survey: OK/NOK) wajib diisi saat masuk stage erfin_diproses".to_string()),
        }));
    }
    if req.stage == "erfin_ready" {
        if req.erfin_number.is_none() || req.erfin_number.as_deref() == Some("") {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("erfin_number (Nomor ERFIN) wajib diisi saat masuk stage erfin_ready".to_string()),
            }));
        }
        if req.erfin_date.is_none() || req.erfin_date.as_deref() == Some("") {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("erfin_date (Tanggal ERFIN) wajib diisi saat masuk stage erfin_ready".to_string()),
            }));
        }
        if req.erfin_ready_date.is_none() || req.erfin_ready_date.as_deref() == Some("") {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("erfin_ready_date (Tanggal ERFIN Ready) wajib diisi saat masuk stage erfin_ready".to_string()),
            }));
        }
    }

    // Validasi permit_date wajib saat masuk permit_process
    if req.stage == "permit_process" && req.permit_date.is_none() {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("permit_date (Tanggal Buat Permit) wajib diisi saat masuk stage permit_process".to_string()),
        }));
    }

    // Validasi field wajib saat transisi ke permit_ready
    if req.stage == "permit_ready" {
        if !content_type.starts_with("multipart/form-data") {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("Stage permit_ready wajib menggunakan multipart/form-data dan upload dokumen TPAS (field file)".to_string()),
            }));
        }
        if permit_doc_file_bytes.is_none() {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("Dokumen TPAS wajib diupload saat masuk stage permit_ready (field file)".to_string()),
            }));
        }
        if !req.tpas_approved.unwrap_or(false) {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("TPAS Approved wajib dicentang untuk masuk stage permit_ready".to_string()),
            }));
        }
        if !req.tp_approved.unwrap_or(false) {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("TP Approved wajib dicentang untuk masuk stage permit_ready".to_string()),
            }));
        }
    }

    // Validasi field wajib saat transisi ke akses_process
    if req.stage == "akses_process" {
        if req.tower_provider.is_none() {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("tower_provider (Tower Provider) wajib diisi saat masuk stage akses_process".to_string()),
            }));
        }
        if req.jenis_kunci.is_none() {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("jenis_kunci (Jenis Kunci) wajib diisi saat masuk stage akses_process".to_string()),
            }));
        }
    }

    // Validasi field wajib saat transisi ke akses_ready
    if req.stage == "akses_ready" {
        if !content_type.starts_with("multipart/form-data") {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("Stage akses_ready wajib menggunakan multipart/form-data".to_string()),
            }));
        }

        if !req.konfirmasi_akses.unwrap_or(false) {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("konfirmasi_akses (Akses ke site sudah READY EKSEKUSI) wajib dicentang".to_string()),
            }));
        }

        if req.has_akses_gedung.unwrap_or(false) {
            if req.gedung_nama.is_none() || req.gedung_nama.as_deref() == Some("") {
                return Ok(Json(ApiResponse {
                    success: false,
                    data: None,
                    message: Some("gedung_nama (Nama Gedung) wajib diisi bila Ada Akses Gedung = Ya".to_string()),
                }));
            }
        }
        
        // Memastikan ada file bukti akses atau minimal sudah ada evidence file
        if multiple_evidence_files.is_empty() {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("Foto kondisi site / bukti akses wajib diupload (field file / files[]) saat masuk stage akses_ready".to_string()),
            }));
        }
    }

    // Validasi field wajib saat transisi ke implementasi
    if req.stage == "implementasi" {
        if req.tgl_rencana_implementasi.is_none() || req.tgl_rencana_implementasi.as_deref() == Some("") {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("tgl_rencana_implementasi (Tanggal Rencana Implementasi) wajib diisi saat masuk stage implementasi".to_string()),
            }));
        }
    }

    // Validasi field wajib saat transisi ke rfi_done
    if req.stage == "rfi_done" {
        if req.jam_checkout.is_none() || req.jam_checkout.as_deref() == Some("") {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("jam_checkout (Jam Check-Out) wajib diisi saat masuk stage rfi_done".to_string()),
            }));
        }

        if !req.konfirmasi_rfi.unwrap_or(false) {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some("konfirmasi_rfi (RFI sudah selesai dilakukan) wajib dicentang".to_string()),
            }));
        }
    }

    // Update stage di sites
    let update_query = "UPDATE type::thing($site_id) SET \
        stage = $stage, \
        stage_updated_at = time::now(), \
        stage_notes = $stage_notes, \
        permit_date = IF $stage = 'permit_process' THEN $permit_date ELSE permit_date END, \
        impl_cico_done = $impl_cico_done, \
        impl_rfs_done = $impl_rfs_done, \
        impl_dokumen_done = $impl_dokumen_done, \
        ineom_registered = $ineom_registered, \
        tpas_approved = IF $stage = 'permit_ready' THEN $tpas_approved ELSE tpas_approved END, \
        tp_approved = IF $stage = 'permit_ready' THEN $tp_approved ELSE tp_approved END, \
        caf_approved = IF $stage = 'permit_ready' THEN $caf_approved ELSE caf_approved END, \
        tgl_berlaku_permit_tpas = IF $stage = 'permit_ready' THEN $tgl_berlaku_permit_tpas ELSE tgl_berlaku_permit_tpas END, \
        tgl_berakhir_permit_tpas = IF $stage = 'permit_ready' THEN $tgl_berakhir_permit_tpas ELSE tgl_berakhir_permit_tpas END, \
        tower_provider = IF $stage = 'akses_process' THEN $tower_provider ELSE tower_provider END, \
        jenis_kunci = IF $stage = 'akses_process' THEN $jenis_kunci ELSE jenis_kunci END, \
        pic_akses_nama = IF $stage = 'akses_process' THEN $pic_akses_nama ELSE pic_akses_nama END, \
        pic_akses_telp = IF $stage = 'akses_process' THEN $pic_akses_telp ELSE pic_akses_telp END, \
        has_akses_gedung = IF $stage = 'akses_ready' THEN $has_akses_gedung ELSE has_akses_gedung END, \
        gedung_nama = IF $stage = 'akses_ready' THEN $gedung_nama ELSE gedung_nama END, \
        gedung_pic_nama = IF $stage = 'akses_ready' THEN $gedung_pic_nama ELSE gedung_pic_nama END, \
        gedung_pic_telp = IF $stage = 'akses_ready' THEN $gedung_pic_telp ELSE gedung_pic_telp END, \
        gedung_akses_status = IF $stage = 'akses_ready' THEN $gedung_akses_status ELSE gedung_akses_status END, \
        konfirmasi_akses = IF $stage = 'akses_ready' THEN $konfirmasi_akses ELSE konfirmasi_akses END, \
        tgl_rencana_implementasi = IF $stage = 'implementasi' THEN $tgl_rencana_implementasi ELSE tgl_rencana_implementasi END, \
        tgl_aktual_mulai = IF $stage = 'implementasi' THEN $tgl_aktual_mulai ELSE tgl_aktual_mulai END, \
        jam_checkin = IF $stage = 'implementasi' THEN $jam_checkin ELSE jam_checkin END, \
        jam_checkout = IF $stage = 'rfi_done' THEN $jam_checkout ELSE jam_checkout END, \
        survey_date = IF $stage = 'survey' THEN $survey_date ELSE survey_date END, \
        survey_result = IF $stage = 'erfin_diproses' THEN $survey_result ELSE survey_result END, \
        survey_nok_reason = IF $stage = 'erfin_diproses' THEN $survey_nok_reason ELSE survey_nok_reason END, \
        erfin_number = IF $stage = 'erfin_ready' THEN $erfin_number ELSE erfin_number END, \
        erfin_date = IF $stage = 'erfin_ready' THEN $erfin_date ELSE erfin_date END, \
        erfin_ready_date = IF $stage = 'erfin_ready' THEN $erfin_ready_date ELSE erfin_ready_date END, \
        konfirmasi_rfi = IF $stage = 'rfi_done' THEN $konfirmasi_rfi ELSE konfirmasi_rfi END, \
        catatan_teknis = IF $stage = 'rfi_done' OR $stage = 'implementasi' THEN $catatan_teknis ELSE catatan_teknis END, \
        updated_at = time::now()";

    let mut update_res = state
        .db
        .query(update_query)
        .bind(("site_id", site_thing.clone()))
        .bind(("stage", req.stage.clone()))
        .bind(("stage_notes", req.notes.clone()))
        .bind(("permit_date", req.permit_date.clone()))
        .bind(("impl_cico_done", req.impl_cico_done.unwrap_or(false)))
        .bind(("impl_rfs_done", req.impl_rfs_done.unwrap_or(false)))
        .bind(("impl_dokumen_done", req.impl_dokumen_done.unwrap_or(false)))
        .bind(("ineom_registered", req.ineom_registered.unwrap_or(false)))
        .bind(("tpas_approved", req.tpas_approved.unwrap_or(false)))
        .bind(("tp_approved", req.tp_approved.unwrap_or(false)))
        .bind(("caf_approved", req.caf_approved.unwrap_or(false)))
        .bind(("tgl_berlaku_permit_tpas", req.tgl_berlaku_permit_tpas.clone()))
        .bind(("tgl_berakhir_permit_tpas", req.tgl_berakhir_permit_tpas.clone()))
        .bind(("tower_provider", req.tower_provider.map(|ep| ep.as_str().to_string())))
        .bind(("jenis_kunci", req.jenis_kunci.map(|ek| ek.as_str().to_string())))
        .bind(("pic_akses_nama", req.pic_akses_nama.clone()))
        .bind(("pic_akses_telp", req.pic_akses_telp.clone()))
        .bind(("has_akses_gedung", req.has_akses_gedung.unwrap_or(false)))
        .bind(("gedung_nama", req.gedung_nama.clone()))
        .bind(("gedung_pic_nama", req.gedung_pic_nama.clone()))
        .bind(("gedung_pic_telp", req.gedung_pic_telp.clone()))
        .bind(("gedung_akses_status", req.gedung_akses_status.clone()))
        .bind(("konfirmasi_akses", req.konfirmasi_akses.unwrap_or(false)))
        .bind(("tgl_rencana_implementasi", req.tgl_rencana_implementasi.clone()))
        .bind(("tgl_aktual_mulai", req.tgl_aktual_mulai.clone()))
        .bind(("jam_checkin", req.jam_checkin.clone()))
        .bind(("jam_checkout", req.jam_checkout.clone()))
        .bind(("konfirmasi_rfi", req.konfirmasi_rfi.unwrap_or(false)))
        .bind(("catatan_teknis", req.catatan_teknis.clone()))
        .bind(("survey_date", req.survey_date.clone()))
        .bind(("survey_result", req.survey_result.clone()))
        .bind(("survey_nok_reason", req.survey_nok_reason.clone()))
        .bind(("erfin_number", req.erfin_number.clone()))
        .bind(("erfin_date", req.erfin_date.clone()))
        .bind(("erfin_ready_date", req.erfin_ready_date.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error updating site stage: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let updated: Vec<Site> = update_res.take(0).map_err(|e| {
        eprintln!("Parse error updating site stage: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut site = updated.into_iter().next().ok_or_else(|| {
        eprintln!("Site not found after stage update");
        StatusCode::NOT_FOUND
    })?;

    if req.stage == "permit_ready" {
        // Gunakan .as_ref() agar permit_doc_file_bytes tidak berpindah (moved),
        // sehingga tetap bisa digunakan oleh blok non-permit_ready di bawah.
        let file_bytes = permit_doc_file_bytes.as_ref().ok_or(StatusCode::BAD_REQUEST)?;
        let filename = permit_doc_filename
            .clone()
            .unwrap_or_else(|| format!("permit_tpas_{}.bin", Utc::now().timestamp()));

        let doc_type_str = permit_doc_type.clone().unwrap_or_else(|| "tpas".to_string());
        let valid_doc_types = ["tpas", "tp", "caf", "lainnya"];
        if !valid_doc_types.contains(&doc_type_str.as_str()) {
            return Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: Some(format!(
                    "doc_type '{}' tidak valid. Gunakan: tpas | tp | caf | lainnya",
                    doc_type_str
                )),
            }));
        }

        let mime_type = permit_doc_content_type
            .clone()
            .filter(|ct| ct != "application/octet-stream" && !ct.is_empty())
            .unwrap_or_else(|| {
                let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
                match ext.as_str() {
                    "pdf" => "application/pdf",
                    "jpg" | "jpeg" => "image/jpeg",
                    "png" => "image/png",
                    "gif" => "image/gif",
                    "webp" => "image/webp",
                    "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                    "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    "zip" => "application/zip",
                    _ => "application/octet-stream",
                }
                .to_string()
            });

        let base64_data = base64::engine::general_purpose::STANDARD.encode(file_bytes);
        let data_url = format!("data:{};base64,{}", mime_type, base64_data);
        let uploaded_by_value = permit_doc_uploaded_by
            .clone()
            .or_else(|| req.changed_by.clone())
            .unwrap_or_else(|| "system".to_string());

        let create_doc_query = "CREATE site_permit_doc SET \
            site_id = $site_id, \
            filename = $filename, \
            original_name = $filename, \
            file_url = $file_url, \
            mime_type = $mime_type, \
            file_size = $file_size, \
            doc_type = $doc_type, \
            uploaded_by = $uploaded_by, \
            uploaded_at = time::now()";

        state
            .db
            .query(create_doc_query)
            .bind(("site_id", site_thing.clone()))
            .bind(("filename", filename))
            .bind(("file_url", data_url))
            .bind(("mime_type", mime_type))
            .bind(("file_size", file_bytes.len() as i64))
            .bind(("doc_type", doc_type_str))
            .bind(("uploaded_by", uploaded_by_value))
            .await
            .map_err(|e| {
                eprintln!("Database error creating permit doc during stage update: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    // ─── SIMPAN FILE YANG DIUPLOAD KE site_evidence / site_files ───────────────
    // Berlaku untuk SEMUA stage (bukan hanya akses_ready/rfi_done/implementasi).
    // Aturan routing berdasarkan MIME type:
    //   image/*  → site_evidence  (foto bukti / progress)
    //   lainnya  → site_files     (dokumen, PDF, dsb.)
    //
    // Bagian 1: Single 'file' field untuk stage selain permit_ready
    if req.stage != "permit_ready" {
        // Gunakan .as_ref() + clone() agar permit_doc_file_bytes tidak di-move,
        // dan semua nilai yang di-bind ke SurrealDB adalah owned ('static).
        if let Some(file_bytes) = permit_doc_file_bytes.as_ref() {
            let filename = permit_doc_filename.clone().unwrap_or_else(|| "upload".to_string());
            if !file_bytes.is_empty() {
                let mime = permit_doc_content_type
                    .clone()
                    .filter(|ct| ct != "application/octet-stream" && !ct.is_empty())
                    .unwrap_or_else(|| {
                        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
                        match ext.as_str() {
                            "pdf"  => "application/pdf",
                            "jpg" | "jpeg" => "image/jpeg",
                            "png"  => "image/png",
                            "gif"  => "image/gif",
                            "webp" => "image/webp",
                            "mp4"  => "video/mp4",
                            "mov"  => "video/quicktime",
                            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                            "zip"  => "application/zip",
                            _      => "application/octet-stream",
                        }.to_string()
                    });
                let b64 = base64::engine::general_purpose::STANDARD.encode(file_bytes.as_slice());
                let data_url = format!("data:{};base64,{}", mime, b64);
                let uploader = permit_doc_uploaded_by
                    .clone()
                    .or_else(|| req.changed_by.clone())
                    .unwrap_or_else(|| "system".to_string());
                let fsize = file_bytes.len() as i64;

                if mime.starts_with("image/") {
                    // → site_evidence (foto)
                    let q = "CREATE site_evidence SET \
                        site_id = $site_id, \
                        filename = $filename, \
                        original_name = $filename, \
                        file_url = $file_url, \
                        mime_type = $mime_type, \
                        file_size = $file_size, \
                        progress_tag = $progress_tag, \
                        stage_context = $stage_context, \
                        uploaded_by = $uploaded_by, \
                        uploaded_at = time::now()";
                    let _ = state.db.query(q)
                        .bind(("site_id", site_thing.clone()))
                        .bind(("filename", filename.clone()))
                        .bind(("file_url", data_url))
                        .bind(("mime_type", mime.clone()))
                        .bind(("file_size", fsize))
                        .bind(("progress_tag", req.stage.clone()))
                        .bind(("stage_context", format!("Foto Evidence – {}", req.stage)))
                        .bind(("uploaded_by", uploader))
                        .await
                        .map_err(|e| eprintln!("⚠️  Failed to save single image to site_evidence: {}", e));
                } else {
                    // → site_files (dokumen)
                    let q = "CREATE site_files SET \
                        site_id = $site_id, \
                        title = $title, \
                        filename = $filename, \
                        original_name = $filename, \
                        file_data = $file_data, \
                        key = $filename, \
                        mime_type = $mime_type, \
                        size = $size, \
                        uploaded_at = time::now(), \
                        created_at = time::now(), \
                        updated_at = time::now()";
                    let _ = state.db.query(q)
                        .bind(("site_id", site_thing.clone()))
                        .bind(("title", filename.clone()))
                        .bind(("filename", filename.clone()))
                        .bind(("file_data", data_url))
                        .bind(("mime_type", mime.clone()))
                        .bind(("size", fsize))
                        .await
                        .map_err(|e| eprintln!("⚠️  Failed to save single file to site_files: {}", e));
                }
            }
        }
    }

    // Bagian 2: Multiple evidence files (evidence_files / files / files[] / bukti_akses)
    // Berlaku untuk SEMUA stage – sebelumnya hanya akses_ready, rfi_done, implementasi
    if !multiple_evidence_files.is_empty() {
        for (filename, mime_type, file_bytes) in multiple_evidence_files {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&file_bytes);
            let data_url = format!("data:{};base64,{}", mime_type, b64);
            let uploaded_by_value = req.changed_by.clone().unwrap_or_else(|| "system".to_string());
            let fsize = file_bytes.len() as i64;

            if mime_type.starts_with("image/") {
                // Image → site_evidence
                let create_evidence_query = "CREATE site_evidence SET \
                    site_id = $site_id, \
                    filename = $filename, \
                    original_name = $filename, \
                    file_url = $file_url, \
                    mime_type = $mime_type, \
                    file_size = $file_size, \
                    progress_tag = $progress_tag, \
                    stage_context = $stage_context, \
                    uploaded_by = $uploaded_by, \
                    uploaded_at = time::now()";
                let _ = state.db.query(create_evidence_query)
                    .bind(("site_id", site_thing.clone()))
                    .bind(("filename", filename.clone()))
                    .bind(("file_url", data_url))
                    .bind(("mime_type", mime_type.clone()))
                    .bind(("file_size", fsize))
                    .bind(("progress_tag", req.stage.clone()))
                    .bind(("stage_context", format!("Foto Evidence – {}", req.stage)))
                    .bind(("uploaded_by", uploaded_by_value))
                    .await
                    .map_err(|e| eprintln!("⚠️  Evidence image upload error: {}", e));
            } else {
                // Non-image → site_files
                let create_file_query = "CREATE site_files SET \
                    site_id = $site_id, \
                    title = $title, \
                    filename = $filename, \
                    original_name = $filename, \
                    file_data = $file_data, \
                    key = $filename, \
                    mime_type = $mime_type, \
                    size = $size, \
                    uploaded_at = time::now(), \
                    created_at = time::now(), \
                    updated_at = time::now()";
                let _ = state.db.query(create_file_query)
                    .bind(("site_id", site_thing.clone()))
                    .bind(("title", filename.clone()))
                    .bind(("filename", filename.clone()))
                    .bind(("file_data", data_url))
                    .bind(("mime_type", mime_type.clone()))
                    .bind(("size", fsize))
                    .await
                    .map_err(|e| eprintln!("⚠️  Site file upload error: {}", e));
            }
        }
    }

    enrich_site_timing_fields(&mut site);

    // Catat log perubahan stage
    let log_query = "CREATE site_stage_log SET \
        site_id = $site_id, \
        from_stage = $from_stage, \
        to_stage = $to_stage, \
        notes = $notes, \
        changed_by = $changed_by, \
        evidence_urls = $evidence_urls, \
        created_at = time::now()";

    let _ = state
        .db
        .query(log_query)
        .bind(("site_id", site_thing))
        .bind(("from_stage", from_stage))
        .bind(("to_stage", req.stage.clone()))
        .bind(("notes", req.notes.clone()))
        .bind(("changed_by", req.changed_by.clone().unwrap_or_else(|| "system".to_string())))
        .bind(("evidence_urls", req.evidence_urls.clone().unwrap_or_default()))
        .await
        .map_err(|e| eprintln!("Warning: failed to create stage log: {}", e));

    Ok(Json(ApiResponse {
        success: true,
        data: Some(site),
        message: Some(format!("Stage berhasil diupdate ke '{}'", req.stage)),
    }))
}

/// GET /api/sites/:id/stage-logs
/// Ambil riwayat perubahan stage untuk satu site
pub async fn get_site_stage_logs(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Vec<SiteStageLog>>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id)?;

    let query = "SELECT * FROM site_stage_log WHERE site_id = $site_id ORDER BY created_at DESC";

    let mut response = state
        .db
        .query(query)
        .bind(("site_id", site_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error fetching stage logs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let logs: Vec<SiteStageLog> = response.take(0).map_err(|e| {
        eprintln!("Parse error fetching stage logs: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(logs),
        message: None,
    }))
}

// ==================== SITE BOQ HANDLERS ====================

/// GET /api/sites/:site_id/boq
pub async fn list_site_boq(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Vec<SiteBoq>>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id)?;

    let mut response = state
        .db
        .query("SELECT * FROM site_boq WHERE site_id = $site_id ORDER BY created_at ASC")
        .bind(("site_id", site_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error listing site boq: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<SiteBoq> = response.take(0).map_err(|e| {
        eprintln!("Parse error listing site boq: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(items),
        message: None,
    }))
}

/// POST /api/sites/:site_id/boq
pub async fn create_site_boq(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
    Json(req): Json<CreateSiteBoqRequest>,
) -> Result<Json<ApiResponse<SiteBoq>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id)?;

    let query = "CREATE site_boq SET \
        site_id = $site_id, \
        item_code = $item_code, \
        description = $description, \
        quantity = $quantity, \
        unit = $unit, \
        type = $boq_type, \
        source = $source, \
        created_at = time::now(), \
        updated_at = time::now()";

    let mut response = state
        .db
        .query(query)
        .bind(("site_id", site_thing))
        .bind(("item_code", req.item_code.clone()))
        .bind(("description", req.description.clone()))
        .bind(("quantity", req.quantity))
        .bind(("unit", req.unit.clone()))
        .bind(("boq_type", req.boq_type.clone().unwrap_or_else(|| "material".to_string())))
        .bind(("source", req.source.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error creating site boq: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<SiteBoq> = response.take(0).map_err(|e| {
        eprintln!("Parse error creating site boq: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let item = items.into_iter().next().ok_or_else(|| {
        eprintln!("No site boq returned after creation");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(item),
        message: Some("BOQ item created successfully".to_string()),
    }))
}

/// PUT /api/site-boq/:boq_id
pub async fn update_site_boq(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(boq_id): axum::extract::Path<String>,
    Json(req): Json<UpdateSiteBoqRequest>,
) -> Result<Json<ApiResponse<SiteBoq>>, StatusCode> {
    let boq_thing = parse_thing_id(&boq_id)?;

    let mut update_parts = vec!["updated_at = time::now()".to_string()];
    if req.item_code.is_some() { update_parts.push("item_code = $item_code".to_string()); }
    if req.description.is_some() { update_parts.push("description = $description".to_string()); }
    if req.quantity.is_some() { update_parts.push("quantity = $quantity".to_string()); }
    if req.unit.is_some() { update_parts.push("unit = $unit".to_string()); }
    if req.boq_type.is_some() { update_parts.push("type = $boq_type".to_string()); }
    if req.source.is_some() { update_parts.push("source = $source".to_string()); }

    let update_query = format!("UPDATE type::thing($boq_id) SET {}", update_parts.join(", "));
    let mut qb = state.db.query(&update_query).bind(("boq_id", boq_thing));

    if let Some(v) = req.item_code { qb = qb.bind(("item_code", v)); }
    if let Some(v) = req.description { qb = qb.bind(("description", v)); }
    if let Some(v) = req.quantity { qb = qb.bind(("quantity", v)); }
    if let Some(v) = req.unit { qb = qb.bind(("unit", v)); }
    if let Some(v) = req.boq_type { qb = qb.bind(("boq_type", v)); }
    if let Some(v) = req.source { qb = qb.bind(("source", v)); }

    let mut response = qb.await.map_err(|e| {
        eprintln!("Database error updating site boq: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let items: Vec<SiteBoq> = response.take(0).map_err(|e| {
        eprintln!("Parse error updating site boq: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let item = items.into_iter().next().ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(item),
        message: Some("BOQ item updated successfully".to_string()),
    }))
}

/// DELETE /api/site-boq/:boq_id
pub async fn delete_site_boq(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(boq_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let boq_thing = parse_thing_id(&boq_id)?;

    state
        .db
        .query("DELETE type::thing($boq_id)")
        .bind(("boq_id", boq_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting site boq: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("BOQ item deleted successfully".to_string()),
    }))
}

// ==================== SKP HANDLERS ====================

/// GET /api/sites/:site_id/skp
pub async fn list_skp_by_site(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Vec<Skp>>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id)?;

    let mut response = state
        .db
        .query("SELECT * FROM skp WHERE site_id = $site_id ORDER BY created_at DESC")
        .bind(("site_id", site_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error listing skp: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<Skp> = response.take(0).map_err(|e| {
        eprintln!("Parse error listing skp: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(items),
        message: None,
    }))
}

/// POST /api/sites/:site_id/skp
pub async fn create_skp(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
    Json(req): Json<CreateSkpRequest>,
) -> Result<Json<ApiResponse<Skp>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id)?;

    let query = "CREATE skp SET \
        site_id = $site_id, \
        skp_number = $skp_number, \
        tanggal = $tanggal, \
        keterangan = $keterangan, \
        status = 'Draft', \
        uploaded_by = $uploaded_by, \
        document_url = $document_url, \
        created_at = time::now(), \
        updated_at = time::now()";

    let mut response = state
        .db
        .query(query)
        .bind(("site_id", site_thing))
        .bind(("skp_number", req.skp_number.clone()))
        .bind(("tanggal", req.tanggal.clone()))
        .bind(("keterangan", req.keterangan.clone()))
        .bind(("uploaded_by", req.uploaded_by.clone()))
        .bind(("document_url", req.document_url.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error creating skp: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<Skp> = response.take(0).map_err(|e| {
        eprintln!("Parse error creating skp: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let item = items.into_iter().next().ok_or_else(|| {
        eprintln!("No skp returned after creation");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(item),
        message: Some("SKP created successfully".to_string()),
    }))
}

/// GET /api/skp/:skp_id
pub async fn get_skp(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(skp_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Skp>>, StatusCode> {
    let skp_thing = parse_thing_id(&skp_id)?;

    let mut response = state
        .db
        .query("SELECT * FROM type::thing($skp_id)")
        .bind(("skp_id", skp_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error getting skp: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<Skp> = response.take(0).map_err(|e| {
        eprintln!("Parse error getting skp: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let item = items.into_iter().next().ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(item),
        message: None,
    }))
}

/// PUT /api/skp/:skp_id
pub async fn update_skp(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(skp_id): axum::extract::Path<String>,
    Json(req): Json<UpdateSkpRequest>,
) -> Result<Json<ApiResponse<Skp>>, StatusCode> {
    let skp_thing = parse_thing_id(&skp_id)?;

    let mut update_parts = vec!["updated_at = time::now()".to_string()];
    if req.skp_number.is_some() { update_parts.push("skp_number = $skp_number".to_string()); }
    if req.tanggal.is_some() { update_parts.push("tanggal = $tanggal".to_string()); }
    if req.keterangan.is_some() { update_parts.push("keterangan = $keterangan".to_string()); }
    if req.status.is_some() { update_parts.push("status = $status".to_string()); }
    if req.document_url.is_some() { update_parts.push("document_url = $document_url".to_string()); }
    if req.received_proof_url.is_some() { update_parts.push("received_proof_url = $received_proof_url".to_string()); }

    let update_query = format!("UPDATE type::thing($skp_id) SET {}", update_parts.join(", "));
    let mut qb = state.db.query(&update_query).bind(("skp_id", skp_thing));

    if let Some(v) = req.skp_number { qb = qb.bind(("skp_number", v)); }
    if let Some(v) = req.tanggal { qb = qb.bind(("tanggal", v)); }
    if let Some(v) = req.keterangan { qb = qb.bind(("keterangan", v)); }
    if let Some(v) = req.status { qb = qb.bind(("status", v)); }
    if let Some(v) = req.document_url { qb = qb.bind(("document_url", v)); }
    if let Some(v) = req.received_proof_url { qb = qb.bind(("received_proof_url", v)); }

    let mut response = qb.await.map_err(|e| {
        eprintln!("Database error updating skp: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let items: Vec<Skp> = response.take(0).map_err(|e| {
        eprintln!("Parse error updating skp: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let item = items.into_iter().next().ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(item),
        message: Some("SKP updated successfully".to_string()),
    }))
}

/// DELETE /api/skp/:skp_id
pub async fn delete_skp(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(skp_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let skp_thing = parse_thing_id(&skp_id)?;

    state
        .db
        .query("DELETE type::thing($skp_id)")
        .bind(("skp_id", skp_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting skp: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("SKP deleted successfully".to_string()),
    }))
}

// ==================== SITE EVIDENCE HANDLERS ====================

/// GET /api/sites/:site_id/evidence
pub async fn list_site_evidence(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Vec<SiteEvidence>>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id)?;

    let mut response = state
        .db
        .query("SELECT * FROM site_evidence WHERE site_id = $site_id ORDER BY uploaded_at DESC")
        .bind(("site_id", site_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error listing site evidence: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<SiteEvidence> = response.take(0).map_err(|e| {
        eprintln!("Parse error listing site evidence: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(items),
        message: None,
    }))
}

/// POST /api/sites/:site_id/evidence  (multipart/form-data)
/// Form fields:
///   - file         : binary (required) — gambar / dokumen
///   - progress_tag : text   (required) — e.g. "implementasi"
///   - stage_context: text   (optional) — keterangan tahap
///   - uploaded_by  : text   (required) — nama / ID uploader
pub async fn create_site_evidence(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<SiteEvidence>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id)?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut file_content_type: Option<String> = None;
    let mut progress_tag: Option<String> = None;
    let mut stage_context: Option<String> = None;
    let mut uploaded_by: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        eprintln!("Multipart field error: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        match field.name().unwrap_or("") {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                file_content_type = field.content_type().map(|s| s.to_string());
                let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                file_data = Some(bytes.to_vec());
            }
            "progress_tag" => {
                progress_tag = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            "stage_context" => {
                stage_context = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            "uploaded_by" => {
                uploaded_by = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            _ => {}
        }
    }

    // Validate required fields
    let file_bytes = file_data.ok_or_else(|| {
        eprintln!("Missing required field: file");
        StatusCode::BAD_REQUEST
    })?;
    let filename = file_name.ok_or_else(|| {
        eprintln!("Missing file name from multipart");
        StatusCode::BAD_REQUEST
    })?;
    let progress_tag_str = progress_tag.ok_or_else(|| {
        eprintln!("Missing required field: progress_tag");
        StatusCode::BAD_REQUEST
    })?;
    let uploaded_by_str = uploaded_by.ok_or_else(|| {
        eprintln!("Missing required field: uploaded_by");
        StatusCode::BAD_REQUEST
    })?;
    // Detect MIME type from file extension if browser did not send Content-Type
    let mime_type = file_content_type
        .filter(|ct| ct != "application/octet-stream" && !ct.is_empty())
        .unwrap_or_else(|| {
            let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
            match ext.as_str() {
                "pdf"  => "application/pdf",
                "jpg" | "jpeg" => "image/jpeg",
                "png"  => "image/png",
                "gif"  => "image/gif",
                "webp" => "image/webp",
                "mp4"  => "video/mp4",
                "mov"  => "video/quicktime",
                "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "zip"  => "application/zip",
                _      => "application/octet-stream",
            }.to_string()
        });
    let file_size = file_bytes.len() as i64;

    // Encode file as base64 data URL for storage
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_bytes);
    let data_url = format!("data:{};base64,{}", mime_type, base64_data);

    let query = "CREATE site_evidence SET \
        site_id = $site_id, \
        filename = $filename, \
        original_name = $filename, \
        file_url = $file_url, \
        mime_type = $mime_type, \
        file_size = $file_size, \
        progress_tag = $progress_tag, \
        stage_context = $stage_context, \
        uploaded_by = $uploaded_by, \
        uploaded_at = time::now()";

    let mut response = state
        .db
        .query(query)
        .bind(("site_id", site_thing))
        .bind(("filename", filename))
        .bind(("file_url", data_url))
        .bind(("mime_type", mime_type))
        .bind(("file_size", file_size))
        .bind(("progress_tag", progress_tag_str))
        .bind(("stage_context", stage_context))
        .bind(("uploaded_by", uploaded_by_str))
        .await
        .map_err(|e| {
            eprintln!("Database error creating site evidence: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<SiteEvidence> = response.take(0).map_err(|e| {
        eprintln!("Parse error creating site evidence: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let item = items.into_iter().next().ok_or_else(|| {
        eprintln!("No site evidence returned after creation");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(item),
        message: Some("Evidence uploaded successfully".to_string()),
    }))
}

/// DELETE /api/site-evidence/:evidence_id
pub async fn delete_site_evidence(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(evidence_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let evidence_thing = parse_thing_id(&evidence_id)?;

    state
        .db
        .query("DELETE type::thing($evidence_id)")
        .bind(("evidence_id", evidence_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting site evidence: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("Evidence deleted successfully".to_string()),
    }))
}

/// GET /api/site-evidence/:evidence_id
/// Returns JSON metadata for a single evidence record.
pub async fn get_site_evidence_by_id(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(evidence_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<SiteEvidence>>, StatusCode> {
    let evidence_thing = parse_thing_id(&evidence_id)?;

    let mut response = state
        .db
        .query("SELECT * FROM type::thing($evidence_id)")
        .bind(("evidence_id", evidence_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error getting site evidence by id: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<SiteEvidence> = response.take(0).map_err(|e| {
        eprintln!("Parse error getting site evidence by id: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let evidence = items.into_iter().next().ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(evidence),
        message: None,
    }))
}

/// GET /api/site-evidence/:evidence_id/preview
/// Decodes the stored base64 data URL and returns raw binary bytes so the
/// browser can render a preview (image inline, PDF inline, etc.).
pub async fn get_site_evidence_preview(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(evidence_id): axum::extract::Path<String>,
) -> Result<axum::response::Response, StatusCode> {
    let evidence_thing = parse_thing_id(&evidence_id)?;

    let mut db_resp = state
        .db
        .query("SELECT * FROM type::thing($evidence_id)")
        .bind(("evidence_id", evidence_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error getting site evidence preview: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<SiteEvidence> = db_resp.take(0).map_err(|e| {
        eprintln!("Parse error getting site evidence preview: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let evidence = items.into_iter().next().ok_or(StatusCode::NOT_FOUND)?;

    let data_url = evidence.file_url.ok_or_else(|| {
        eprintln!("Evidence has no file_url");
        StatusCode::NOT_FOUND
    })?;

    // Parse "data:{mime_type};base64,{encoded_data}"
    let after_data = data_url.strip_prefix("data:").ok_or_else(|| {
        eprintln!("Evidence file_url is not a valid data URL");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;
    let semi_pos = after_data.find(';').ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
    let mime_type = after_data[..semi_pos].to_string();
    let b64_part = &after_data[semi_pos + 1..];
    let b64_data = b64_part.strip_prefix("base64,").ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64_data)
        .map_err(|e| {
            eprintln!("Base64 decode error for evidence preview: {}", e);
            StatusCode::UNPROCESSABLE_ENTITY
        })?;

    let filename = evidence.original_name.unwrap_or(evidence.filename);

    let response = axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, &mime_type)
        .header(
            axum::http::header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", filename),
        )
        .body(axum::body::Body::from(bytes))
        .map_err(|e| {
            eprintln!("Failed to build preview response: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(response)
}

// ==================== SITE PERMIT DOCUMENT HANDLERS ====================

/// GET /api/sites/:site_id/permit-docs
/// List semua dokumen izin yang sudah diupload untuk satu site.
pub async fn list_site_permit_docs(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Vec<SitePermitDoc>>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id)?;

    let mut response = state
        .db
        .query("SELECT * FROM site_permit_doc WHERE site_id = $site_id ORDER BY uploaded_at DESC")
        .bind(("site_id", site_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error listing site permit docs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<SitePermitDoc> = response.take(0).map_err(|e| {
        eprintln!("Parse error listing site permit docs: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse { success: true, data: Some(items), message: None }))
}

/// POST /api/sites/:site_id/permit-docs  (multipart/form-data)
/// Form fields:
///   - file        : binary (required)
///   - doc_type    : text   (required) — "tpas" | "tp" | "caf" | "lainnya"
///   - uploaded_by : text   (required)
pub async fn create_site_permit_doc(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<SitePermitDoc>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id)?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut file_content_type: Option<String> = None;
    let mut doc_type: Option<String> = None;
    let mut uploaded_by: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        eprintln!("Multipart field error: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        match field.name().unwrap_or("") {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                file_content_type = field.content_type().map(|s| s.to_string());
                let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                file_data = Some(bytes.to_vec());
            }
            "doc_type" => {
                doc_type = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            "uploaded_by" => {
                uploaded_by = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
            }
            _ => {}
        }
    }

    let file_bytes = file_data.ok_or_else(|| {
        eprintln!("Missing required field: file");
        StatusCode::BAD_REQUEST
    })?;
    let filename = file_name.ok_or_else(|| {
        eprintln!("Missing file name from multipart");
        StatusCode::BAD_REQUEST
    })?;
    let doc_type_str = doc_type.ok_or_else(|| {
        eprintln!("Missing required field: doc_type");
        StatusCode::BAD_REQUEST
    })?;
    let uploaded_by_str = uploaded_by.ok_or_else(|| {
        eprintln!("Missing required field: uploaded_by");
        StatusCode::BAD_REQUEST
    })?;

    // Validate doc_type value
    let valid_doc_types = ["tpas", "tp", "caf", "lainnya"];
    if !valid_doc_types.contains(&doc_type_str.as_str()) {
        eprintln!("Invalid doc_type: {}", doc_type_str);
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    // Detect MIME type from file extension if browser did not send Content-Type
    let mime_type = file_content_type
        .filter(|ct| ct != "application/octet-stream" && !ct.is_empty())
        .unwrap_or_else(|| {
            let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
            match ext.as_str() {
                "pdf"  => "application/pdf",
                "jpg" | "jpeg" => "image/jpeg",
                "png"  => "image/png",
                "gif"  => "image/gif",
                "webp" => "image/webp",
                "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "zip"  => "application/zip",
                _      => "application/octet-stream",
            }.to_string()
        });
    let file_size = file_bytes.len() as i64;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&file_bytes);
    let data_url = format!("data:{};base64,{}", mime_type, base64_data);

    let query = "CREATE site_permit_doc SET \
        site_id = $site_id, \
        filename = $filename, \
        original_name = $filename, \
        file_url = $file_url, \
        mime_type = $mime_type, \
        file_size = $file_size, \
        doc_type = $doc_type, \
        uploaded_by = $uploaded_by, \
        uploaded_at = time::now()";

    let mut db_resp = state
        .db
        .query(query)
        .bind(("site_id", site_thing))
        .bind(("filename", filename))
        .bind(("file_url", data_url))
        .bind(("mime_type", mime_type))
        .bind(("file_size", file_size))
        .bind(("doc_type", doc_type_str))
        .bind(("uploaded_by", uploaded_by_str))
        .await
        .map_err(|e| {
            eprintln!("Database error creating site permit doc: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<SitePermitDoc> = db_resp.take(0).map_err(|e| {
        eprintln!("Parse error creating site permit doc: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let item = items.into_iter().next().ok_or_else(|| {
        eprintln!("No site permit doc returned after creation");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(item),
        message: Some("Dokumen permit berhasil diupload".to_string()),
    }))
}

/// GET /api/permit-docs/:doc_id
/// Ambil metadata satu dokumen permit berdasarkan ID.
pub async fn get_site_permit_doc_by_id(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<SitePermitDoc>>, StatusCode> {
    let doc_thing = parse_thing_id(&doc_id)?;

    let mut response = state
        .db
        .query("SELECT * FROM type::thing($doc_id)")
        .bind(("doc_id", doc_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error getting site permit doc by id: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<SitePermitDoc> = response.take(0).map_err(|e| {
        eprintln!("Parse error getting site permit doc by id: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let doc = items.into_iter().next().ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ApiResponse { success: true, data: Some(doc), message: None }))
}

/// GET /api/permit-docs/:doc_id/preview
/// Serve raw binary agar bisa dipreview langsung di browser.
pub async fn get_site_permit_doc_preview(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
) -> Result<axum::response::Response, StatusCode> {
    let doc_thing = parse_thing_id(&doc_id)?;

    let mut db_resp = state
        .db
        .query("SELECT * FROM type::thing($doc_id)")
        .bind(("doc_id", doc_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error getting site permit doc preview: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<SitePermitDoc> = db_resp.take(0).map_err(|e| {
        eprintln!("Parse error getting site permit doc preview: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let doc = items.into_iter().next().ok_or(StatusCode::NOT_FOUND)?;

    let data_url = doc.file_url.ok_or_else(|| {
        eprintln!("Permit doc has no file_url");
        StatusCode::NOT_FOUND
    })?;

    let after_data = data_url.strip_prefix("data:").ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
    let semi_pos = after_data.find(';').ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
    let mime_type = after_data[..semi_pos].to_string();
    let b64_part = &after_data[semi_pos + 1..];
    let b64_data = b64_part.strip_prefix("base64,").ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64_data)
        .map_err(|e| {
            eprintln!("Base64 decode error for permit doc preview: {}", e);
            StatusCode::UNPROCESSABLE_ENTITY
        })?;

    let filename = doc.original_name.unwrap_or(doc.filename);

    let response = axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, &mime_type)
        .header(
            axum::http::header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", filename),
        )
        .body(axum::body::Body::from(bytes))
        .map_err(|e| {
            eprintln!("Failed to build permit doc preview response: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(response)
}

/// DELETE /api/permit-docs/:doc_id
pub async fn delete_site_permit_doc(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(doc_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let doc_thing = parse_thing_id(&doc_id)?;

    state
        .db
        .query("DELETE type::thing($doc_id)")
        .bind(("doc_id", doc_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting site permit doc: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("Dokumen permit berhasil dihapus".to_string()),
    }))
}

// ==================== SITE ISSUE HANDLERS ====================

/// GET /api/sites/:site_id/issues
pub async fn list_site_issues(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<Vec<SiteIssue>>>, StatusCode> {
    let site_thing = Thing::try_from(site_id.as_str()).map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut response = state
        .db
        .query("SELECT * FROM site_issue WHERE site_id = $site_id ORDER BY created_at DESC")
        .bind(("site_id", site_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error listing site issues: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let issues: Vec<SiteIssue> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(issues),
        message: None,
    }))
}

/// POST /api/sites/:site_id/issues
pub async fn create_site_issue(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
    Json(req): Json<CreateSiteIssueRequest>,
) -> Result<Json<ApiResponse<SiteIssue>>, StatusCode> {
    let site_thing = Thing::try_from(site_id.as_str()).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Validate tindakan value
    if req.tindakan != "tahan" && req.tindakan != "eskalasi" {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("Tindakan harus 'tahan' atau 'eskalasi'".to_string()),
        }));
    }

    let initial_status = if req.tindakan == "eskalasi" { "escalated" } else { "open" };

    let query = "CREATE site_issue SET \
        site_id = $site_id, \
        stage_at_report = $stage_at_report, \
        keterangan = $keterangan, \
        tindakan = $tindakan, \
        status = $status, \
        reported_by = $reported_by, \
        evidence_urls = $evidence_urls, \
        created_at = time::now(), \
        updated_at = time::now()";

    let mut response = state
        .db
        .query(query)
        .bind(("site_id", site_thing))
        .bind(("stage_at_report", req.stage_at_report.clone()))
        .bind(("keterangan", req.keterangan.clone()))
        .bind(("tindakan", req.tindakan.clone()))
        .bind(("status", initial_status.to_string()))
        .bind(("reported_by", req.reported_by.clone()))
        .bind(("evidence_urls", req.evidence_urls.clone().unwrap_or_default()))
        .await
        .map_err(|e| {
            eprintln!("Database error creating site issue: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut issues: Vec<SiteIssue> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match issues.pop() {
        Some(issue) => Ok(Json(ApiResponse {
            success: true,
            data: Some(issue),
            message: Some(format!("Issue dilaporkan. Tindakan: {}", req.tindakan)),
        })),
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// GET /api/site-issues/:issue_id
pub async fn get_site_issue(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(issue_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<SiteIssue>>, StatusCode> {
    let issue_thing = Thing::try_from(issue_id.as_str()).map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut response = state
        .db
        .query("SELECT * FROM type::thing($issue_id)")
        .bind(("issue_id", issue_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error getting site issue: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut issues: Vec<SiteIssue> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match issues.pop() {
        Some(issue) => Ok(Json(ApiResponse {
            success: true,
            data: Some(issue),
            message: None,
        })),
        None => Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("Issue tidak ditemukan".to_string()),
        })),
    }
}

/// POST /api/site-issues/:issue_id/resolve
pub async fn resolve_site_issue(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(issue_id): axum::extract::Path<String>,
    Json(req): Json<ResolveSiteIssueRequest>,
) -> Result<Json<ApiResponse<SiteIssue>>, StatusCode> {
    let issue_thing = Thing::try_from(issue_id.as_str()).map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = "UPDATE type::thing($issue_id) SET \
        status = 'resolved', \
        resolved_by = $resolved_by, \
        resolved_notes = $resolved_notes, \
        resolved_at = time::now(), \
        updated_at = time::now()";

    let mut response = state
        .db
        .query(query)
        .bind(("issue_id", issue_thing))
        .bind(("resolved_by", req.resolved_by.clone()))
        .bind(("resolved_notes", req.resolved_notes.clone()))
        .await
        .map_err(|e| {
            eprintln!("Database error resolving site issue: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut issues: Vec<SiteIssue> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match issues.pop() {
        Some(issue) => Ok(Json(ApiResponse {
            success: true,
            data: Some(issue),
            message: Some("Issue berhasil di-resolve".to_string()),
        })),
        None => Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("Issue tidak ditemukan".to_string()),
        })),
    }
}

/// DELETE /api/site-issues/:issue_id
pub async fn delete_site_issue(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(issue_id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let issue_thing = Thing::try_from(issue_id.as_str()).map_err(|_| StatusCode::BAD_REQUEST)?;

    state
        .db
        .query("DELETE type::thing($issue_id)")
        .bind(("issue_id", issue_thing))
        .await
        .map_err(|e| {
            eprintln!("Database error deleting site issue: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("Issue berhasil dihapus".to_string()),
    }))
}
