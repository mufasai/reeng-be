/// ==================== RBAC PERMISSIONS MODULE ====================
/// Role-Based Access Control dengan granular permission matrix
/// Sesuai dengan mockup frontend permission definitions

use serde::{Deserialize, Serialize};

// ─── USER ROLES ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    #[serde(rename = "director")]
    Director,
    #[serde(rename = "operational")]
    Operational,
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "finance")]
    Finance,
    #[serde(rename = "field")]
    Field,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            UserRole::Director => "director",
            UserRole::Operational => "operational",
            UserRole::Admin => "admin",
            UserRole::Finance => "finance",
            UserRole::Field => "field",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "director" => Some(UserRole::Director),
            "operational" => Some(UserRole::Operational),
            "admin" => Some(UserRole::Admin),
            "finance" => Some(UserRole::Finance),
            "field" => Some(UserRole::Field),
            _ => None,
        }
    }
}

// ─── PERMISSION SYSTEM ───────────────────────────────────────────────────────
/// Granular permission matrix matching mockup frontend
pub struct PermissionMatrix;

impl PermissionMatrix {
    /// Check apakah role memiliki permission tertentu
    pub fn can(role: &UserRole, permission: &str) -> bool {
        match permission {
            // ─── DASHBOARD ────────────────────────────────────────────────
            "dashboard.view" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance | UserRole::Field
            ),
            "dashboard.status_lapangan" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance | UserRole::Field
            ),
            "dashboard.financial_kpi" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
            ),
            "dashboard.butuh_tindakan" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance | UserRole::Field
            ),
            "dashboard.pengajuan_review" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
            ),
            "dashboard.peta_sites" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance | UserRole::Field
            ),
            "dashboard.aktivitas_terbaru" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
            ),

            // ─── SITES ────────────────────────────────────────────────────
            "site.view_list" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
            ),
            "site.view_import_history" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
            ),
            "site.view_detail" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
            ),
            "site.update_stage" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
            ),
            "site.bulk_update" => matches!(role, UserRole::Director | UserRole::Operational | UserRole::Admin),
            "site.import_boq" => matches!(role, UserRole::Director | UserRole::Operational | UserRole::Admin),
            "site.import_review" => matches!(role, UserRole::Operational | UserRole::Admin),
            "site.edit_data" => matches!(role, UserRole::Director | UserRole::Operational | UserRole::Admin),
            "site.delete" => matches!(role, UserRole::Operational),
            "site.assign_team" => matches!(role, UserRole::Operational | UserRole::Admin),

            // ─── STAGE-SPECIFIC ───────────────────────────────────────────
            "stage.imported_to_assigned" => matches!(role, UserRole::Operational | UserRole::Admin),
            "stage.assigned_to_permit" => matches!(role, UserRole::Operational | UserRole::Admin),
            "stage.permit_to_akses" => matches!(role, UserRole::Operational | UserRole::Admin),
            "stage.akses_to_implementasi" => matches!(role, UserRole::Operational | UserRole::Admin),
            "stage.update_cico_rfi_rfs" => matches!(role, UserRole::Operational | UserRole::Admin),
            "stage.dokumen_to_bast" => matches!(role, UserRole::Operational | UserRole::Admin),
            "stage.bast_to_invoice" => matches!(role, UserRole::Operational | UserRole::Admin | UserRole::Finance),
            "stage.report_issue" => matches!(role, UserRole::Operational | UserRole::Admin),

            // ─── MATERIAL ──────────────────────────────────────────────────
            "material.view" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
            ),
            "material.add" => matches!(role, UserRole::Operational | UserRole::Admin),
            "material.edit_status" => matches!(role, UserRole::Operational | UserRole::Admin),

            // ─── FINANCIAL ────────────────────────────────────────────────
            "financial.view_termin_status" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
            ),
            "financial.view_rp_amounts" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
            ),
            "financial.submit_pengajuan" => matches!(role, UserRole::Operational | UserRole::Admin),
            "financial.approve_pengajuan" => matches!(role, UserRole::Director | UserRole::Finance),
            "financial.reject_pengajuan" => matches!(role, UserRole::Director | UserRole::Finance),
            "financial.process_payment" => matches!(role, UserRole::Finance),

            // ─── PEOPLE & TEAMS ───────────────────────────────────────────
            "people.view" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
            ),
            "people.create" => matches!(role, UserRole::Operational | UserRole::Admin),
            "people.edit" => matches!(role, UserRole::Operational | UserRole::Admin),
            "people.delete" => matches!(role, UserRole::Operational),
            "people.import" => matches!(role, UserRole::Admin),
            "team.view" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin
            ),
            "team.create" => matches!(role, UserRole::Operational | UserRole::Admin),
            "team.edit" => matches!(role, UserRole::Operational | UserRole::Admin),
            "team.assign_people" => matches!(role, UserRole::Operational | UserRole::Admin),

            // ─── WORK ORDERS ───────────────────────────────────────────────
            "workorder.view" => matches!(
                role,
                UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
            ),
            "workorder.create" => matches!(role, UserRole::Operational | UserRole::Admin),
            "workorder.edit" => matches!(role, UserRole::Operational | UserRole::Admin),
            "workorder.assign_team" => matches!(role, UserRole::Operational | UserRole::Admin),
            "workorder.create_skp" => matches!(role, UserRole::Operational | UserRole::Admin),

            // ─── ADMIN & SETTINGS ──────────────────────────────────────────
            "admin.user_management" => matches!(role, UserRole::Admin),
            "admin.view_logs" => matches!(role, UserRole::Admin),
            "admin.system_settings" => matches!(role, UserRole::Admin),

            // Default: deny
            _ => false,
        }
    }

    /// Batch check multiple permissions (AND logic)
    pub fn can_all(role: &UserRole, permissions: &[&str]) -> bool {
        permissions.iter().all(|p| Self::can(role, p))
    }

    /// Batch check multiple permissions (OR logic)
    pub fn can_any(role: &UserRole, permissions: &[&str]) -> bool {
        permissions.iter().any(|p| Self::can(role, p))
    }
}

// ─── FIELD-LEVEL RBAC (untuk restricted viewing) ───────────────────────────
/// Role field bisa hanya lihat sites yang assigned ke team mereka
pub fn should_restrict_to_team(role: &UserRole) -> bool {
    matches!(role, UserRole::Field)
}

/// Role finance dan director pun lihat semua
pub fn can_view_all_sites(role: &UserRole) -> bool {
    matches!(
        role,
        UserRole::Director | UserRole::Operational | UserRole::Admin | UserRole::Finance
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_director_permissions() {
        let director = UserRole::Director;
        assert!(PermissionMatrix::can(&director, "dashboard.view"));
        assert!(PermissionMatrix::can(&director, "financial.approve_pengajuan"));
        assert!(PermissionMatrix::can(&director, "admin.user_management") == false);
    }

    #[test]
    fn test_field_restrictions() {
        let field = UserRole::Field;
        assert!(PermissionMatrix::can(&field, "dashboard.view"));
        assert!(!PermissionMatrix::can(&field, "site.bulk_update"));
        assert!(!PermissionMatrix::can(&field, "financial.approve_pengajuan"));
    }

    #[test]
    fn test_can_all() {
        let admin = UserRole::Admin;
        assert!(PermissionMatrix::can_all(
            &admin,
            &["site.view_list", "site.edit_data"]
        ));
        assert!(!PermissionMatrix::can_all(
            &admin,
            &["site.view_list", "admin.user_management"]
        ));
    }

    #[test]
    fn test_role_string_conversion() {
        assert_eq!(UserRole::Director.as_str(), "director");
        assert_eq!(UserRole::from_str("admin"), Some(UserRole::Admin));
        assert_eq!(UserRole::from_str("invalid"), None);
    }
}
