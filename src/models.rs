use serde::{Deserialize, Serialize};

// ==================== AUTH MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user: Option<UserInfo>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub email: String,
    pub nama: String,
    pub role: String,
}

// ==================== PROJECT MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectType {
    Combat,
    #[serde(rename = "L2H")]
    L2h,
    #[serde(rename = "BLACK SITE")]
    BlackSite,
    Refinen,
    Filter,
    #[serde(rename = "BEBAN OPERASIONAL")]
    BebanOperasional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonnelInfo {
    pub id: Option<String>,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSite {
    pub id: Option<String>,
    pub site_name: String,
    pub site_info: String,
    pub pekerjaan: String,
    pub lokasi: String,
    pub nomor_kontrak: String,
    pub start: String,
    pub end: String,
    pub maximal_budget: f64,
    pub cost_estimated: f64,
    pub pemberi_tugas: String,
    pub penerima_tugas: String,
    pub site_document: Option<String>,
    pub teams: Vec<PersonnelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub project_name: String,
    pub lokasi: String,
    pub budget: f64,
    pub tipe: ProjectType,
    pub keterangan: String,
    pub project_document: Option<String>,
    pub sites: Vec<ProjectSite>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub project_name: String,
    pub lokasi: String,
    pub budget: f64,
    pub tipe: ProjectType,
    pub keterangan: String,
    pub project_document: Option<String>,
    pub sites: Vec<ProjectSite>,
}

// ==================== RESPONSE WRAPPER ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}
