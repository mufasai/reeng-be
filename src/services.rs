/// ==================== STAGE SERVICE MODULE ====================
/// Business logic untuk stage transitions, validations, dan status tracking
/// Handles kompleks workflow logic terpisah dari HTTP handlers

use crate::config::{self, StageRequirements};
use crate::common::{validate_stage, validate_termin_ke};
use chrono::{NaiveDate, Utc};

// ─── STAGE TRANSITION SERVICE ─────────────────────────────────────────────────
pub struct StageTransitionService;

impl StageTransitionService {
    /// Validate stage transition dengan semua business rules
    /// Used by site stage update handler untuk enforce workflow
    #[allow(dead_code)]
    pub fn validate_transition(
        current_stage: &str,
        next_stage: &str,
        project_type: &str,
        required_fields_data: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        // Step 1: Check stage validity
        if !validate_stage(current_stage) {
            return Err(format!("Invalid current stage: {}", current_stage));
        }

        if !validate_stage(next_stage) {
            return Err(format!("Invalid next stage: {}", next_stage));
        }

        // Step 2: Check stage transition rules
        config::validate_stage_transition(current_stage, next_stage, project_type)?;

        // Step 3: Check required fields untuk transition ini
        if let Some(requirements) = config::get_stage_requirements(current_stage, next_stage, project_type) {
            validate_required_fields(&requirements, required_fields_data)?;
        }

        Ok(())
    }

    /// Calculate stage metadata (days_in_stage, days_remaining for permit, dll)    /// Reserved untuk dashboard metrics (Phase 2)
    #[allow(dead_code)]    pub fn calculate_stage_metadata(
        current_stage: &str,
        stage_updated_at: &str,
        permit_date: Option<&str>,
    ) -> StageMetadata {
        let stage_updated = NaiveDate::parse_from_str(stage_updated_at, "%Y-%m-%d");
        let today = chrono::Local::now().naive_local().date();

        let days_in_stage = stage_updated
            .ok()
            .and_then(|date| {
                let duration = today.signed_duration_since(date);
                Some(duration.num_days())
            });

        // Calculate permit-related timing (if in permit stages)
        let permit_metrics = if matches!(current_stage, "permit_process" | "permit_ready" | "akses_process") {
            permit_date
                .and_then(|pd| NaiveDate::parse_from_str(pd, "%Y-%m-%d").ok())
                .map(|permit_date| {
                    let total_days = today.signed_duration_since(permit_date).num_days();
                    PermitMetrics {
                        permit_days_total: Some(total_days),
                        permit_days_elapsed: Some(total_days),
                        permit_days_remaining: None,
                    }
                })
        } else {
            None
        };

        StageMetadata {
            days_in_stage,
            permit_metrics,
        }
    }

    /// Get suggested next stages based on current stage
    /// Helper untuk frontend UI dropdown (Phase 2)
    #[allow(dead_code)]
    pub fn get_valid_next_stages(current_stage: &str, project_type: &str) -> Vec<String> {
        let allowed = config::get_allowed_stages_for_project_type(project_type);
        allowed
            .iter()
            .position(|&s| s == current_stage)
            .and_then(|idx| {
                if idx + 1 < allowed.len() {
                    Some(vec![allowed[idx + 1].to_string()])
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    /// Create stage transition log entry
    /// Backup implementation untuk audit tracking (Phase 2)
    #[allow(dead_code)]
    pub fn create_transition_log(
        previous_stage: &str,
        new_stage: &str,
        user_id: String,
        notes: Option<String>,
    ) -> SiteStageLogEntry {
        SiteStageLogEntry {
            from_stage: previous_stage.to_string(),
            to_stage: new_stage.to_string(),
            transitioned_by: user_id,
            transitioned_at: Utc::now(),
            notes,
        }
    }
}

// ─── TERMIN SERVICE ───────────────────────────────────────────────────────────
/// Handles termin (billing) workflow and validations
/// Currently documented for future implementation
pub struct TerminService;

impl TerminService {
    /// Validate termin dapat dibuat berdasarkan stage & termin sebelumnya
    /// NOTE: Implemented for termin workflow (Phase 2)
    #[allow(dead_code)]
    pub fn validate_termin_creation(
        current_stage: &str,
        termin_ke: i32,
    ) -> Result<(), String> {
        // Check termin_ke valid
        // Validate termin_ke range (must be 1-6)
        validate_termin_ke(termin_ke)?;

        // Get termin config
        let config = config::get_termin_config(termin_ke)
            .ok_or(format!("Invalid termin_ke: {}", termin_ke))?;

        // Check apakah site sudah di minimum stage untuk termin ini
        let allowed_stages = config::STAGE_ORDER;
        let current_idx = allowed_stages
            .iter()
            .position(|&s| s == current_stage)
            .ok_or("Invalid current stage".to_string())?;
        let min_idx = allowed_stages
            .iter()
            .position(|&s| s == config.minimum_stage)
            .ok_or("Invalid minimum stage configuration".to_string())?;

        if current_idx < min_idx {
            return Err(format!(
                "Site must reach {} stage before requesting {} (currently at {})",
                config.minimum_stage, config.name, current_stage
            ));
        }

        Ok(())
    }

    /// Check apakah termin sebelumnya sudah approved
    /// Validate previous termins sudah approved sebelum termin baru bisa dibuat
    #[allow(dead_code)]
    pub fn validate_termin_dependency(previous_termins: &[&TerminStatus]) -> Result<(), String> {
        for termin in previous_termins {
            if termin.status != "approved" && termin.status != "paid" {
                return Err(format!(
                    "Previous termin {} must be approved first",
                    termin.termin_name
                ));
            }
        }
        Ok(())
    }

    /// Calculate termin amount berdasarkan site budget & percentage
    /// Calculate termin amount: site_budget × (percentage ÷ 100)
    #[allow(dead_code)]
    pub fn calculate_termin_amount(site_budget: i64, percentage: i32) -> Option<i64> {
        if percentage < 1 || percentage > 100 {
            return None;
        }
        Some((site_budget * percentage as i64) / 100)
    }

    /// Get termin display name
    /// Get display name for termin number (untuk UI)
    #[allow(dead_code)]
    pub fn get_termin_display_name(termin_ke: i32) -> String {
        config::get_termin_name(termin_ke)
    }
}

// ─── DATA STRUCTURES ──────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
/// Stage metadata untuk dashboard metrics (future use)
#[allow(dead_code)]
pub struct StageMetadata {
    pub days_in_stage: Option<i64>,
    pub permit_metrics: Option<PermitMetrics>,
}

#[derive(Debug, Clone)]
/// Permit tracking metrics (future use)
#[allow(dead_code)]
pub struct PermitMetrics {
    pub permit_days_total: Option<i64>,
    pub permit_days_elapsed: Option<i64>,
    pub permit_days_remaining: Option<i64>,
}

#[derive(Debug, Clone)]
/// Stage transition log entry (future use for audit trail)
#[allow(dead_code)]
pub struct SiteStageLogEntry {
    pub from_stage: String,
    pub to_stage: String,
    pub transitioned_by: String,
    pub transitioned_at: chrono::DateTime<chrono::Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
/// Termin payment status tracking (future use)
#[allow(dead_code)]
pub struct TerminStatus {
    pub termin_ke: i32,
    pub termin_name: String,
    pub status: String,
    pub percentage: i32,
}

// ─── VALIDATION HELPERS ───────────────────────────────────────────────────────
/// Validate bahwa semua required fields hadir
#[allow(dead_code)]
fn validate_required_fields(
    requirements: &StageRequirements,
    data: &std::collections::HashMap<String, serde_json::Value>,
) -> Result<(), String> {
    for field in &requirements.required_fields {
        if !data.contains_key(field) || data[field].is_null() {
            return Err(format!(
                "Required field '{}' is missing for {}: {}",
                field, requirements.description, requirements.description
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_calculate_termin_amount() {
        let amount = TerminService::calculate_termin_amount(1_000_000, 30);
        assert_eq!(amount, Some(300_000));

        let invalid = TerminService::calculate_termin_amount(1_000_000, 150);
        assert_eq!(invalid, None);
    }

    #[test]
    fn test_validate_stage_transition() {
        let data = HashMap::new();
        let result = StageTransitionService::validate_transition(
            "imported",
            "assigned",
            "FILTER",
            &data,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_valid_next_stages() {
        let next = StageTransitionService::get_valid_next_stages("imported", "FILTER");
        assert_eq!(next, vec!["assigned".to_string()]);
    }

    #[test]
    fn test_termin_display_name() {
        assert_eq!(TerminService::get_termin_display_name(1), "T1");
        assert_eq!(TerminService::get_termin_display_name(2), "T2a");
    }
}
