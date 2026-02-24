use axum::{extract::Json, http::StatusCode};
use std::sync::Arc;
use surrealdb::sql::Thing;

use crate::models::{ApiResponse, CreateSiteRequest, UpdateSiteRequest, Site, Team};
use crate::state::AppState;

pub async fn create_site(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<CreateSiteRequest>,
) -> Result<Json<ApiResponse<Site>>, StatusCode> {
    // Build query to create site with proper record references
    let query = "CREATE sites CONTENT {
        project_id: type::thing($project_id),
        site_name: $site_name,
        site_info: $site_info,
        pekerjaan: $pekerjaan,
        lokasi: $lokasi,
        nomor_kontrak: $nomor_kontrak,
        start: $start,
        end: $end,
        maximal_budget: $maximal_budget,
        cost_estimated: $cost_estimated,
        pemberi_tugas: $pemberi_tugas,
        penerima_tugas: $penerima_tugas,
        site_document: $site_document,
        created_at: time::now(),
        updated_at: time::now()
    }";

    let mut response = state
        .db
        .query(query)
        .bind(("project_id", req.project_id.clone()))
        .bind(("site_name", req.site_name.clone()))
        .bind(("site_info", req.site_info.clone()))
        .bind(("pekerjaan", req.pekerjaan.clone()))
        .bind(("lokasi", req.lokasi.clone()))
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
            let team_query = "CREATE teams CONTENT {
                nama: $nama,
                project_id: type::thing($project_id),
                site_id: type::thing($site_id),
                active: true,
                created_at: time::now(),
                updated_at: time::now()
            }";

            let mut team_response = state
                .db
                .query(team_query)
                .bind(("nama", format!("Team {}", req.site_name)))
                .bind(("project_id", req.project_id.clone()))
                .bind(("site_id", site_id_str.clone()))
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
                for people_id_str in member_ids {
                    let member_query = "CREATE team_peoples CONTENT {
                        team_id: type::thing($team_id),
                        people_id: type::thing($people_id),
                        created_at: time::now(),
                        updated_at: time::now()
                    }";

                    let _ = state
                        .db
                        .query(member_query)
                        .bind(("team_id", team_id_str.clone()))
                        .bind(("people_id", people_id_str.clone()))
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
        update_parts.push("project_id = type::thing($project_id)".to_string());
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
        query_builder = query_builder.bind(("project_id", project_id));
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
