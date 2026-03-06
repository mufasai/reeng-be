use axum::{extract::Json, http::StatusCode};
use std::sync::Arc;
use surrealdb::sql::Thing;

use crate::models::{ApiResponse, CreateSiteRequest, UpdateSiteRequest, Site, Team, SiteTeamMember, SiteTeamMemberDetail, AddSiteTeamMemberRequest, UpdateSiteTeamMemberRequest, TeamMasterInfo};
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
