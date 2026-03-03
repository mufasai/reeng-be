use axum::{extract::{Json, Path, State}, http::StatusCode};
use std::sync::Arc;
use surrealdb::sql::Thing;

use crate::models::{ApiResponse, CreateTeamRequest, Team, TeamPeople, UpdateTeamRequest};
use crate::state::AppState;

pub async fn create_team(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTeamRequest>,
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
    let teams: Vec<Team> = state
        .db
        .select("teams")
        .await
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

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

pub async fn update_team(
    State(state): State<Arc<AppState>>,
    Path(team_id): Path<String>,
    Json(req): Json<UpdateTeamRequest>,
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
    let mut updates = Vec::new();
    
    if let Some(nama) = req.nama {
        updates.push(format!("nama = '{}'", nama.replace("'", "''")));
    }
    if let Some(project_id) = req.project_id {
        let project_id_clean = strip_table_prefix(&project_id, "projects");
        updates.push(format!("project_id = projects:{}", project_id_clean));
    }
    if let Some(site_id) = req.site_id {
        let site_id_clean = strip_table_prefix(&site_id, "sites");
        updates.push(format!("site_id = sites:{}", site_id_clean));
    }
    if let Some(leader_id) = req.leader_id {
        let leader_id_clean = strip_table_prefix(&leader_id, "people");
        updates.push(format!("leader_id = people:{}", leader_id_clean));
    }
    if let Some(active) = req.active {
        updates.push(format!("active = {}", active));
    }

    if updates.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    updates.push("updated_at = time::now()".to_string());

    let query = format!(
        "UPDATE $team_id SET {}",
        updates.join(", ")
    );

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
