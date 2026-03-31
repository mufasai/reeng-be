/// ==================== COMMON UTILITIES MODULE ====================
/// Reusable functions untuk error handling, ID parsing, validation, dll
/// Clean code patterns untuk konsistensi di seluruh aplikasi

use axum::http::StatusCode;
use serde_json::json;
use surrealdb::sql::Thing;

// ─── Error Response Helpers ──────────────────────────────────────────────────
/// Format error response dengan message yang konsisten
pub fn error_response(status: StatusCode, message: &str) -> (StatusCode, String) {
    (
        status,
        serde_json::to_string(&json!({
            "success": false,
            "message": message,
            "data": None::<String>
        }))
        .unwrap_or_else(|_| "{\"success\": false, \"message\": \"Internal error\"}".to_string()),
    )
}

// ─── ID Parsing & Validation ─────────────────────────────────────────────────
/// Parse string ID dan convert ke Thing reference
/// Menangani format: "table:id" atau hanya "id"
pub fn parse_thing_id(id_str: &str, table: &str) -> Result<Thing, StatusCode> {
    let clean_id = strip_table_prefix(id_str, table);
    Thing::try_from((table, clean_id)).map_err(|_| StatusCode::BAD_REQUEST)
}

/// Strip table prefix dari ID string jika ada
/// Contoh: "sites:site-123" → "site-123"
/// Contoh: "site-123" → "site-123"
pub fn strip_table_prefix<'a>(id_str: &'a str, table: &str) -> &'a str {
    let prefix = format!("{}:", table);
    id_str.strip_prefix(&prefix).unwrap_or(id_str)
}

/// Konversi Thing reference ke string representation
pub fn thing_to_string(thing: &Option<Thing>) -> String {
    thing
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_default()
}

// ─── Validation Helpers ──────────────────────────────────────────────────────
/// Validate required field tidak kosong
pub fn validate_required(value: &Option<String>, field_name: &str) -> Result<(), String> {
    match value {
        Some(v) if !v.trim().is_empty() => Ok(()),
        _ => Err(format!("{} is required", field_name)),
    }
}

/// Validate email format
pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

/// Validate password strength (minimal 8 chars)
pub fn validate_password(password: &str) -> bool {
    password.len() >= 8
}

/// Validate stage adalah valid
pub fn validate_stage(stage: &str) -> bool {
    matches!(
        stage,
        "imported"
            | "assigned"
            | "permit_process"
            | "permit_ready"
            | "akses_process"
            | "akses_ready"
            | "implementasi"
            | "rfi_done"
            | "rfs_done"
            | "dokumen_done"
            | "bast"
            | "invoice"
            | "completed"
            | "issue_hold"
            | "survey"
            | "survey_nok"
            | "erfin_process"
            | "erfin_ready"
    )
}

/// Validate percentage range (1-100)
pub fn validate_percentage(percentage: i32) -> Result<(), String> {
    if percentage >= 1 && percentage <= 100 {
        Ok(())
    } else {
        Err(format!(
            "Percentage harus antara 1-100, got: {}",
            percentage
        ))
    }
}

/// Validate termin_ke range (1-6 untuk support T1-T4 dengan splits)
pub fn validate_termin_ke(termin_ke: i32) -> Result<(), String> {
    if termin_ke >= 1 && termin_ke <= 6 {
        Ok(())
    } else {
        Err(format!(
            "Termin ke harus antara 1-6, got: {}",
            termin_ke
        ))
    }
}

/// Validate jumlah (amount) tidak boleh negatif dan tidak 0
pub fn validate_amount(amount: i64) -> Result<(), String> {
    if amount > 0 {
        Ok(())
    } else {
        Err(format!("Amount harus positive, got: {}", amount))
    }
}

// ─── Date/Time Helpers ───────────────────────────────────────────────────────
/// Calculate days between two dates (ISO format)
pub fn calculate_days_between(start_iso: &str, end_iso: &str) -> Option<i64> {
    use chrono::NaiveDate;

    let start = NaiveDate::parse_from_str(start_iso, "%Y-%m-%d").ok()?;
    let end = NaiveDate::parse_from_str(end_iso, "%Y-%m-%d").ok()?;

    let duration = end.signed_duration_since(start);
    Some(duration.num_days())
}

/// Get current date dalam format ISO (YYYY-MM-DD)
pub fn today_iso() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Get current datetime dalam format ISO 8601
pub fn now_iso() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

// ─── JSON Response Helpers ───────────────────────────────────────────────────
/// Create success JSON response dengan data
pub fn success_json<T: serde::Serialize>(data: T, message: Option<&str>) -> serde_json::Value {
    json!({
        "success": true,
        "data": data,
        "message": message
    })
}

/// Create error JSON response
pub fn error_json(message: &str) -> serde_json::Value {
    json!({
        "success": false,
        "data": None::<String>,
        "message": message
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_table_prefix() {
        assert_eq!(strip_table_prefix("sites:123", "sites"), "123");
        assert_eq!(strip_table_prefix("123", "sites"), "123");
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com"));
        assert!(!validate_email("invalid-email"));
    }

    #[test]
    fn test_validate_percentage() {
        assert!(validate_percentage(50).is_ok());
        assert!(validate_percentage(0).is_err());
        assert!(validate_percentage(101).is_err());
    }

    #[test]
    fn test_validate_stage() {
        assert!(validate_stage("imported"));
        assert!(validate_stage("completed"));
        assert!(!validate_stage("invalid_stage"));
    }
}
