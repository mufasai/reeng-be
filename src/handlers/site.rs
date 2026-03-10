use axum::{extract::Json, http::StatusCode};
use std::sync::Arc;
use surrealdb::sql::Thing;

use crate::models::{
    ApiResponse, CreateSiteRequest, UpdateSiteRequest, UpdateSiteStageRequest,
    Site, SiteStageLog, Team, SiteTeamMember, SiteTeamMemberDetail,
    AddSiteTeamMemberRequest, UpdateSiteTeamMemberRequest, TeamMasterInfo,
    SiteBoq, CreateSiteBoqRequest, UpdateSiteBoqRequest,
    Skp, CreateSkpRequest, UpdateSkpRequest,
    SiteEvidence, CreateSiteEvidenceRequest,
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

    let sites: Vec<Site> = response.take(0).map_err(|e| {
        eprintln!("Parse error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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

// Helper function to parse "table:id" string into Thing
fn parse_thing_id(id_str: &str) -> Result<Thing, StatusCode> {
    // Use from_string to parse Thing from "table:id" format
    Thing::try_from(id_str).map_err(|_| {
        eprintln!("Failed to parse Thing from '{}'", id_str);
        StatusCode::BAD_REQUEST
    })
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

    let updated_site = updated_sites.into_iter().next().ok_or_else(|| {
        eprintln!("No site returned after update");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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
        .bind(("site_id", site_thing))
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
        message: Some("Team member added to site structure successfully".to_string()),
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
    Json(req): Json<UpdateSiteStageRequest>,
) -> Result<Json<ApiResponse<Site>>, StatusCode> {
    let valid_stages = [
        "imported", "assigned", "permit_process", "permit_ready",
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

    // Update stage di sites
    let update_query = "UPDATE type::thing($site_id) SET \
        stage = $stage, \
        stage_updated_at = time::now(), \
        stage_notes = $stage_notes, \
        impl_cico_done = $impl_cico_done, \
        impl_rfs_done = $impl_rfs_done, \
        impl_dokumen_done = $impl_dokumen_done, \
        ineom_registered = $ineom_registered, \
        updated_at = time::now()";

    let mut update_res = state
        .db
        .query(update_query)
        .bind(("site_id", site_thing.clone()))
        .bind(("stage", req.stage.clone()))
        .bind(("stage_notes", req.notes.clone()))
        .bind(("impl_cico_done", req.impl_cico_done.unwrap_or(false)))
        .bind(("impl_rfs_done", req.impl_rfs_done.unwrap_or(false)))
        .bind(("impl_dokumen_done", req.impl_dokumen_done.unwrap_or(false)))
        .bind(("ineom_registered", req.ineom_registered.unwrap_or(false)))
        .await
        .map_err(|e| {
            eprintln!("Database error updating site stage: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let updated: Vec<Site> = update_res.take(0).map_err(|e| {
        eprintln!("Parse error updating site stage: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let site = updated.into_iter().next().ok_or_else(|| {
        eprintln!("Site not found after stage update");
        StatusCode::NOT_FOUND
    })?;

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

/// POST /api/sites/:site_id/evidence
pub async fn create_site_evidence(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(site_id): axum::extract::Path<String>,
    Json(req): Json<CreateSiteEvidenceRequest>,
) -> Result<Json<ApiResponse<SiteEvidence>>, StatusCode> {
    let site_thing = parse_thing_id(&site_id)?;

    let query = "CREATE site_evidence SET \
        site_id = $site_id, \
        filename = $filename, \
        original_name = $original_name, \
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
        .bind(("filename", req.filename.clone()))
        .bind(("original_name", req.original_name.clone()))
        .bind(("file_url", req.file_url.clone()))
        .bind(("mime_type", req.mime_type.clone()))
        .bind(("file_size", req.file_size))
        .bind(("progress_tag", req.progress_tag.clone()))
        .bind(("stage_context", req.stage_context.clone()))
        .bind(("uploaded_by", req.uploaded_by.clone()))
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
