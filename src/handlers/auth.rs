use crate::extractors::FormOrJson;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use std::sync::Arc;
use surrealdb::sql::Thing;
use crate::models::{LoginRequest, LoginResponse, RegisterRequest, UserInfo, User, ApiResponse, UpdateUserRequest};
use crate::state::AppState;

// ==================== REGISTER ====================

pub async fn register(
    State(state): State<Arc<AppState>>,
    FormOrJson(req): FormOrJson<RegisterRequest>,
) -> Result<Json<ApiResponse<UserInfo>>, StatusCode> {
    // Validate email format
    if !req.email.contains('@') {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("Invalid email format".to_string()),
        }));
    }

    // Check if email already exists
    let check_query = "SELECT * FROM users WHERE email = $email";
    let mut check_result = state.db.query(check_query)
        .bind(("email", req.email.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let existing_user: Option<User> = check_result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing_user.is_some() {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("Email already registered".to_string()),
        }));
    }

    // Hash password (simple hash for demo - use bcrypt in production)
    let password_hash = format!("hashed_{}", req.password);

    // Convert role enum to string
    let role_str = match req.role {
        crate::models::UserRole::BackofficeAdmin => "backoffice admin",
        crate::models::UserRole::Management => "management",
        crate::models::UserRole::TeamLeader => "team leader",
        crate::models::UserRole::HeadOffice => "head office",
        crate::models::UserRole::Finance => "finance",
        crate::models::UserRole::Engineer => "engineer",
        crate::models::UserRole::Admin => "admin",
        crate::models::UserRole::Direktur => "direktur",
    };

    // Create user
    let create_query = r#"
        CREATE users CONTENT {
            name: $name,
            email: $email,
            password: $password,
            role: $role,
            email_verified_at: NONE,
            remember_token: NONE,
            created_at: time::now(),
            updated_at: time::now()
        }
    "#;

    let mut result = state.db.query(create_query)
        .bind(("name", req.name.clone()))
        .bind(("email", req.email.clone()))
        .bind(("password", password_hash))
        .bind(("role", role_str))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user: Option<User> = result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(user) => {
            let user_info = UserInfo {
                id: user.id.as_ref().map(|id| id.to_string()).unwrap_or_default(),
                name: user.name,
                email: user.email,
                role: user.role,
            };

            Ok(Json(ApiResponse {
                success: true,
                data: Some(user_info),
                message: Some("User registered successfully".to_string()),
            }))
        }
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ==================== LOGIN ====================

pub async fn login(
    State(state): State<Arc<AppState>>,
    FormOrJson(req): FormOrJson<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Find user by email
    let query = "SELECT * FROM users WHERE email = $email LIMIT 1";
    let mut result = state.db.query(query)
        .bind(("email", req.email.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user: Option<User> = result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(user) => {
            // Verify password (simple check for demo - use bcrypt in production)
            let password_hash = format!("hashed_{}", req.password);
            
            if user.password != password_hash {
                return Ok(Json(LoginResponse {
                    success: false,
                    token: None,
                    user: None,
                    message: Some("Invalid credentials".to_string()),
                }));
            }

            // Generate token
            let token = format!("token_{}_{}", user.email, chrono::Utc::now().timestamp());

            let user_info = UserInfo {
                id: user.id.as_ref().map(|id| id.to_string()).unwrap_or_default(),
                name: user.name,
                email: user.email,
                role: user.role,
            };

            Ok(Json(LoginResponse {
                success: true,
                token: Some(token),
                user: Some(user_info),
                message: Some("Login successful".to_string()),
            }))
        }
        None => Ok(Json(LoginResponse {
            success: false,
            token: None,
            user: None,
            message: Some("Invalid credentials".to_string()),
        })),
    }
}

// ==================== USER MANAGEMENT ====================

pub async fn get_all_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<User>>>, StatusCode> {
    let query = "SELECT * FROM users ORDER BY created_at DESC";
    
    let mut result = state.db.query(query)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users: Vec<User> = result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(users),
        message: None,
    }))
}

pub async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<ApiResponse<User>>, StatusCode> {
    let thing = Thing::try_from(("users", user_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = "SELECT * FROM $user_id";
    let mut result = state.db.query(query)
        .bind(("user_id", thing))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user: Option<User> = result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(user) => Ok(Json(ApiResponse {
            success: true,
            data: Some(user),
            message: None,
        })),
        None => Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("User not found".to_string()),
        })),
    }
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    FormOrJson(req): FormOrJson<UpdateUserRequest>,
) -> Result<Json<ApiResponse<User>>, StatusCode> {
    let thing = Thing::try_from(("users", user_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Build dynamic update query
    let mut updates = Vec::new();
    
    if req.name.is_some() {
        updates.push("name = $name");
    }
    if req.email.is_some() {
        updates.push("email = $email");
    }
    if req.role.is_some() {
        updates.push("role = $role");
    }
    if req.password.is_some() {
        updates.push("password = $password");
    }

    if updates.is_empty() {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("No fields to update".to_string()),
        }));
    }

    updates.push("updated_at = time::now()");
    let update_fields = updates.join(", ");
    let query = format!("UPDATE $user_id SET {}", update_fields);

    let mut db_query = state.db.query(&query)
        .bind(("user_id", thing));

    if let Some(name) = req.name {
        db_query = db_query.bind(("name", name));
    }
    if let Some(email) = req.email {
        db_query = db_query.bind(("email", email));
    }
    if let Some(role) = req.role {
        db_query = db_query.bind(("role", role));
    }
    if let Some(password) = req.password {
        let password_hash = format!("hashed_{}", password);
        db_query = db_query.bind(("password", password_hash));
    }

    let mut result = db_query
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user: Option<User> = result.take(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(user) => Ok(Json(ApiResponse {
            success: true,
            data: Some(user),
            message: Some("User updated successfully".to_string()),
        })),
        None => Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: Some("User not found".to_string()),
        })),
    }
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let thing = Thing::try_from(("users", user_id.as_str()))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let query = "DELETE $user_id";
    state.db.query(query)
        .bind(("user_id", thing))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        message: Some("User deleted successfully".to_string()),
    }))
}
