use axum::{extract::Json, http::StatusCode};
use crate::models::{LoginRequest, LoginResponse, UserInfo};

pub async fn login(Json(req): Json<LoginRequest>) -> Result<Json<LoginResponse>, StatusCode> {
    // Simple hardcoded login untuk testing
    if req.email == "admin@smartelco.com" && req.password == "admin123" {
        let user = UserInfo {
            email: req.email,
            nama: "Admin SmartElco".to_string(),
            role: "ADMIN".to_string(),
        };

        // Generate simple JWT (untuk production, gunakan proper JWT)
        let token = format!("token_{}_{}", user.email, chrono::Utc::now().timestamp());

        Ok(Json(LoginResponse {
            success: true,
            token: Some(token),
            user: Some(user),
            message: Some("Login successful".to_string()),
        }))
    } else {
        Ok(Json(LoginResponse {
            success: false,
            token: None,
            user: None,
            message: Some("Invalid credentials".to_string()),
        }))
    }
}
