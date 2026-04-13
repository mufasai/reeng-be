/// ==================== CONFIGURATION MODULE ====================
/// Centralized configuration untuk stages, project types, termin requirements
/// Sesuai dengan mockup frontend untuk konsistensi bisnis logic

use serde::{Deserialize, Serialize};

// ─── STAGE DEFINITIONS ───────────────────────────────────────────────────────
/// Order dan hierarchy dari stages (menentukan validasi transisi)
pub const STAGE_ORDER: &[&str] = &[
    "imported",
    "assigned",
    "survey",
    "erfin_diproses",
    "erfin_ready",
    "permit_process",
    "permit_ready",
    "akses_process",
    "akses_ready",
    "implementasi",
    "rfi_done",
    "rfs_done",
    "dokumen_done",
    "bast",
    "invoice",
    "completed",
];

/// Stage yang merupakan milestone (untuk reporting)
pub const MILESTONE_STAGES: &[&str] = &["assigned", "permit_ready", "akses_ready", "implementasi", "completed"];

/// Stages untuk project type tertentu
pub fn get_allowed_stages_for_project_type(_project_type: &str) -> Vec<&str> {
    STAGE_ORDER.to_vec()
}

// ─── TERMIN CONFIGURATIONS ───────────────────────────────────────────────────
/// Termin ke mapping (1-6 untuk support split payments)
/// T1 = 1 (30%), T2a = 2 (15%), T2b = 3 (25%), T2c = 4 (10%), T3 = 5 (10%), T4 = 6 (10%)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminConfig {
    pub termin_ke: i32,
    pub name: &'static str,
    pub percentage: i32,
    pub requires_skp: bool,
    pub requires_bast: bool,
    pub minimum_stage: &'static str,
}

pub const TERMIN_CONFIGS: &[TerminConfig] = &[
    TerminConfig {
        termin_ke: 1,
        name: "T1",
        percentage: 30,
        requires_skp: true,
        requires_bast: false,
        minimum_stage: "permit_ready",
    },
    TerminConfig {
        termin_ke: 2,
        name: "T2a",
        percentage: 15,
        requires_skp: false,
        requires_bast: false,
        minimum_stage: "akses_ready",
    },
    TerminConfig {
        termin_ke: 3,
        name: "T2b",
        percentage: 25,
        requires_skp: false,
        requires_bast: false,
        minimum_stage: "implementasi",
    },
    TerminConfig {
        termin_ke: 4,
        name: "T2c",
        percentage: 10,
        requires_skp: false,
        requires_bast: false,
        minimum_stage: "rfi_done",
    },
    TerminConfig {
        termin_ke: 5,
        name: "T3",
        percentage: 10,
        requires_skp: false,
        requires_bast: true,
        minimum_stage: "bast",
    },
    TerminConfig {
        termin_ke: 6,
        name: "T4",
        percentage: 10,
        requires_skp: false,
        requires_bast: true,
        minimum_stage: "invoice",
    },
];

/// Get termin config by termin_ke
pub fn get_termin_config(termin_ke: i32) -> Option<&'static TerminConfig> {
    TERMIN_CONFIGS.iter().find(|c| c.termin_ke == termin_ke)
}

/// Get termin name untuk display (T1, T2a, T2b, T2c, T3, T4)
pub fn get_termin_name(termin_ke: i32) -> String {
    get_termin_config(termin_ke)
        .map(|c| c.name.to_string())
        .unwrap_or_else(|| format!("T{}", termin_ke))
}

// ─── STAGE TRANSITION RULES ──────────────────────────────────────────────────
/// Tentukan apakah transisi stage valid
/// Rules: max +1 step forward, atau skip dengan valid reasons
pub fn validate_stage_transition(
    current_stage: &str,
    next_stage: &str,
    project_type: &str,
) -> Result<(), String> {
    // Get allowed stages untuk project type
    let allowed = get_allowed_stages_for_project_type(project_type);

    // Check next_stage ada di allowed stages
    if !allowed.contains(&next_stage) {
        return Err(format!(
            "Stage {} tidak valid untuk project type {}",
            next_stage, project_type
        ));
    }

    // Get index dalam STAGE_ORDER
    let current_idx = allowed.iter().position(|&s| s == current_stage);
    let next_idx = allowed.iter().position(|&s| s == next_stage);

    match (current_idx, next_idx) {
        (Some(curr), Some(next)) => {
            if next > curr && next <= curr + 1 {
                Ok(()) // Valid: forward by 1
            } else if next == curr {
                Err("Cannot update to same stage".to_string())
            } else if next < curr {
                Err(format!(
                    "Cannot move backward from {} to {}",
                    current_stage, next_stage
                ))
            } else {
                Err(format!(
                    "Cannot skip stages from {} to {}. Max +1 step allowed.",
                    current_stage, next_stage
                ))
            }
        }
        _ => Err(format!(
            "Invalid stage transition: {} → {}",
            current_stage, next_stage
        )),
    }
}

// ─── REQUIRED FIELDS PER STAGE ───────────────────────────────────────────────
/// Define required fields untuk setiap stage transition
/// Ini akan digunakan saat update stage untuk validasi data yang perlu di-fill

pub struct StageRequirements {
    pub required_fields: Vec<String>,
    pub description: String,
}

pub fn get_stage_requirements(
    from_stage: &str,
    to_stage: &str,
    _project_type: &str,
) -> Option<StageRequirements> {
    match (from_stage, to_stage) {
        // IMPORTED → ASSIGNED
        ("imported", "assigned") => Some(StageRequirements {
            required_fields: vec!["team_id".to_string()],
            description: "Assign tim untuk mengerjakan site".to_string(),
        }),

        // ASSIGNED → SURVEY
        ("assigned", "survey") => Some(StageRequirements {
            required_fields: vec!["survey_date".to_string()],
            description: "Catat tanggal survei lapangan".to_string(),
        }),
        
        // SURVEY → ERFIN_DIPROSES
        ("survey", "erfin_diproses") => Some(StageRequirements {
            required_fields: vec!["survey_result".to_string()],
            description: "Isi hasil survey OK atau NOK".to_string(),
        }),

        // ERFIN_DIPROSES → ERFIN_READY
        ("erfin_diproses", "erfin_ready") => Some(StageRequirements {
            required_fields: vec!["erfin_number".to_string(), "erfin_date".to_string(), "erfin_ready_date".to_string()],
            description: "Input nomor erfin, tanggal erfin, dan tanggal erfin ready".to_string(),
        }),

        // ERFIN_READY → PERMIT_PROCESS
        ("erfin_ready", "permit_process") => Some(StageRequirements {
            required_fields: vec!["permit_date".to_string()],
            description: "Input tanggal buat permit".to_string(),
        }),

        // PERMIT_PROCESS → PERMIT_READY
        ("permit_process", "permit_ready") => Some(StageRequirements {
            required_fields: vec![
                "approval_chain".to_string(),
                "tpas_approved".to_string(),
                "tp_approved".to_string(),
                "caf_approved".to_string(),
                "tgl_berlaku_permit_tpas".to_string(),
                "tgl_berakhir_permit_tpas".to_string(),
                "dokumen_tpas_url".to_string(),
            ],
            description:
                "Input Approval Chain, TPAS, TP, CAF, dan upload dokumen".to_string(),
        }),

        // PERMIT_READY → AKSES_PROCESS
        ("permit_ready", "akses_process") => Some(StageRequirements {
            required_fields: vec![
                "tower_provider".to_string(),
                "jenis_kunci".to_string(),
                "pic_akses_nama".to_string(),
                "pic_akses_telp".to_string(),
            ],
            description: "Input informasi akses tower untuk tim lapangan".to_string(),
        }),

        // AKSES_PROCESS → AKSES_READY
        ("akses_process", "akses_ready") => Some(StageRequirements {
            required_fields: vec![], // Dikonfirm bahwa akses sudah siap
            description: "Konfirmasi akses tower sudah siap untuk implementasi".to_string(),
        }),

        // AKSES_READY → IMPLEMENTASI
        ("akses_ready", "implementasi") => Some(StageRequirements {
            required_fields: vec!["tgl_rencana_implementasi".to_string()],
            description: "Set tanggal rencana implementasi".to_string(),
        }),

        // IMPLEMENTASI → RFI_DONE
        ("implementasi", "rfi_done") => Some(StageRequirements {
            required_fields: vec!["jam_checkin".to_string()],
            description: "Catat jam check-in implementasi".to_string(),
        }),

        // RFI_DONE → RFS_DONE
        ("rfi_done", "rfs_done") => Some(StageRequirements {
            required_fields: vec!["jam_checkout".to_string()],
            description: "Catat jam check-out implementasi".to_string(),
        }),

        // RFS_DONE → DOKUMEN_DONE
        ("rfs_done", "dokumen_done") => Some(StageRequirements {
            required_fields: vec!["impl_dokumen_done".to_string()],
            description: "Dokumen implementasi sudah complete dan ter-submit".to_string(),
        }),

        // DOKUMEN_DONE → BAST
        ("dokumen_done", "bast") => Some(StageRequirements {
            required_fields: vec!["ineom_registered".to_string()],
            description: "BAST sudah ditandatangani dan INEOM ter-register".to_string(),
        }),

        // BAST → INVOICE
        ("bast", "invoice") => Some(StageRequirements {
            required_fields: vec![], // Invoice dibuat otomatis
            description: "Invoice ter-generate dan siap untuk pembayaran".to_string(),
        }),

        // INVOICE → COMPLETED
        ("invoice", "completed") => Some(StageRequirements {
            required_fields: vec![],
            description: "Site sudah completed dan all payments received".to_string(),
        }),

        _ => None,
    }
}

// ─── ROLE-BASED STAGE TRANSITION PERMISSIONS ─────────────────────────────────
/// Tentukan role mana yang bisa trigger stage transition
pub fn can_transition_stage(role: &str, stage_transition: &str) -> bool {
    match (role, stage_transition) {
        // Operational dan Admin bisa trigger transisi apapun
        ("operational", _) | ("admin", _) => true,
        // Director bisa approve
        ("director", _) => true,
        // Finance officer hanya bisa approve termin-related
        ("finance", "bast" | "invoice" | "completed") => true,
        // Field engineer bisa approve implementasi stages
        ("field", "implementasi" | "rfi_done" | "rfs_done") => true,
        // Lainnya tidak bisa
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_stage_transition_forward() {
        let result = validate_stage_transition("imported", "assigned", "FILTER");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_stage_transition_skip() {
        let result = validate_stage_transition("imported", "permit_process", "FILTER");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_termin_config() {
        let cfg = get_termin_config(1);
        assert!(cfg.is_some());
        assert_eq!(cfg.unwrap().percentage, 30);
    }

    #[test]
    fn test_stage_order_valid() {
        assert!(STAGE_ORDER.len() > 10);
        assert_eq!(STAGE_ORDER[0], "imported");
        assert_eq!(STAGE_ORDER[STAGE_ORDER.len() - 1], "completed");
    }
}
