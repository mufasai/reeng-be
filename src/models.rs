use serde::{Deserialize, Serialize, Serializer};
use surrealdb::sql::Thing;

// ==================== CUSTOM SERIALIZERS ====================

// Custom serializer for Thing to display as "table:id" string format
mod thing_serializer {
    use super::*;
    
    pub fn serialize<S>(thing: &Option<Thing>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match thing {
            Some(thing) => serializer.serialize_str(&thing.to_string()),
            None => serializer.serialize_none(),
        }
    }
}

// ==================== AUTH MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    #[serde(rename = "backoffice admin")]
    BackofficeAdmin,
    Management,
    #[serde(rename = "team leader")]
    TeamLeader,
    #[serde(rename = "head office")]
    HeadOffice,
    Finance,
    Engineer,
    Admin,
    Direktur,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
}

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
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

// ==================== ENUMS ====================

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
    #[serde(rename = "OSP")]
    Osp,
}

// ==================== PEOPLE MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct People {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    pub name: String,
    pub tanggal_lahir: Option<String>,
    pub tempat_lahir: Option<String>,
    pub agama: Option<String>,
    pub jenis_kelamin: Option<String>,
    pub no_ktp: Option<String>,
    pub no_hp: Option<String>,
    pub email: Option<String>,
    pub jabatan_kerja: Option<String>,
    pub regional: Option<String>,
    pub lokasi_kerja: Option<String>,
    pub pekerjaan: Option<String>,
    pub nama_kontak_darurat: Option<String>,
    pub nomor_kontak_darurat: Option<String>,
    pub alamat_kontak_darurat: Option<String>,
    pub status_pernikahan: Option<String>,
    pub nama_ibu_kandung: Option<String>,
    pub pendidikan_terakhir: Option<String>,
    pub nama_kampus_sekolah: Option<String>,
    pub jurusan_sekolah: Option<String>,
    pub tahun_lulus: Option<String>,
    pub foto_ktp: Option<String>,
    pub foto_diri: Option<String>,
    pub thumbnail_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePeopleRequest {
    pub name: String,
    pub tanggal_lahir: Option<String>,
    pub tempat_lahir: Option<String>,
    pub agama: Option<String>,
    pub jenis_kelamin: Option<String>,
    pub no_ktp: Option<String>,
    pub no_hp: Option<String>,
    pub email: Option<String>,
    pub jabatan_kerja: Option<String>,
    pub regional: Option<String>,
    pub lokasi_kerja: Option<String>,
    pub pekerjaan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePeopleRequest {
    pub name: Option<String>,
    pub tanggal_lahir: Option<String>,
    pub tempat_lahir: Option<String>,
    pub agama: Option<String>,
    pub jenis_kelamin: Option<String>,
    pub no_ktp: Option<String>,
    pub no_hp: Option<String>,
    pub email: Option<String>,
    pub jabatan_kerja: Option<String>,
    pub regional: Option<String>,
    pub lokasi_kerja: Option<String>,
    pub pekerjaan: Option<String>,
    pub nama_kontak_darurat: Option<String>,
    pub nomor_kontak_darurat: Option<String>,
    pub alamat_kontak_darurat: Option<String>,
    pub status_pernikahan: Option<String>,
    pub nama_ibu_kandung: Option<String>,
    pub pendidikan_terakhir: Option<String>,
    pub nama_kampus_sekolah: Option<String>,
    pub jurusan_sekolah: Option<String>,
    pub tahun_lulus: Option<String>,
}

// ==================== PROJECT MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    pub name: String,  // Changed from project_name
    pub lokasi: String,
    pub value: i64,  // Changed from budget - this is the project value/anggaran
    pub cost: i64,   // NEW - actual cost spent
    pub keterangan: String,
    pub tipe: ProjectType,
    pub tgi_start: Option<String>,  // NEW - tanggal mulai
    pub tgi_end: Option<String>,    // NEW - tanggal selesai
    pub status: Option<String>,     // NEW
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub lokasi: String,
    pub value: i64,
    pub cost: Option<i64>,
    pub tipe: ProjectType,
    pub keterangan: String,
    pub tgi_start: Option<String>,
    pub tgi_end: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub lokasi: Option<String>,
    pub value: Option<i64>,
    pub cost: Option<i64>,
    pub tipe: Option<ProjectType>,
    pub keterangan: Option<String>,
    pub tgi_start: Option<String>,
    pub tgi_end: Option<String>,
    pub status: Option<String>,
}

// ==================== SITE MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub project_id: Option<Thing>,
    pub site_name: String,
    pub site_info: String,
    pub pekerjaan: String,
    pub lokasi: String,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub nomor_kontrak: String,
    pub start: String,
    pub end: String,
    pub maximal_budget: i64,
    pub cost_estimated: i64,
    pub pemberi_tugas: String,
    pub penerima_tugas: String,
    pub site_document: Option<String>,
    // Stage tracking
    pub stage: Option<String>,              // imported | assigned | permit_process | permit_ready | akses_process | akses_ready | implementasi | rfi_done | rfs_done | dokumen_done | bast | invoice | completed
    pub stage_updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days_in_stage: Option<i64>,
    pub stage_notes: Option<String>,
    pub permit_date: Option<String>,        // Tanggal buat permit (diisi saat masuk permit_process)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_days_total: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_days_elapsed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_days_remaining: Option<i64>,
    pub impl_cico_done: Option<bool>,
    pub impl_rfs_done: Option<bool>,
    pub impl_dokumen_done: Option<bool>,
    pub ineom_registered: Option<bool>,
    // Permit ready stage data (diisi saat transisi → permit_ready)
    pub tpas_approved: Option<bool>,
    pub tp_approved: Option<bool>,
    pub caf_approved: Option<bool>,
    pub tgl_berlaku_permit_tpas: Option<String>,
    pub tgl_berakhir_permit_tpas: Option<String>,
    // Akses process stage data (diisi saat transisi → akses_process)
    pub tower_provider: Option<String>,     // MITRATEL | STP | PTI | DMT | Lainnya
    pub jenis_kunci: Option<String>,        // PADLOCK | SMARTLOCK | QUADLOCK
    pub pic_akses_nama: Option<String>,
    pub pic_akses_telp: Option<String>,
    // Implementasi stage data (diisi saat transisi → implementasi)
    pub tgl_rencana_implementasi: Option<String>,
    pub tgl_aktual_mulai: Option<String>,
    pub jam_checkin: Option<String>,
    // RFI done stage data (diisi saat transisi → rfi_done)
    pub jam_checkout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSiteRequest {
    pub project_id: String,  // Will be converted to Thing
    pub site_name: String,
    pub site_info: String,
    pub pekerjaan: String,
    pub lokasi: String,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub nomor_kontrak: String,
    pub start: String,
    pub end: String,
    pub maximal_budget: i64,
    pub cost_estimated: i64,
    pub pemberi_tugas: String,
    pub penerima_tugas: String,
    pub site_document: Option<String>,
    pub team_members: Option<Vec<String>>,  // Array of people IDs for the team
    pub stage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSiteRequest {
    pub project_id: Option<String>,
    pub site_name: Option<String>,
    pub site_info: Option<String>,
    pub pekerjaan: Option<String>,
    pub lokasi: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub nomor_kontrak: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub maximal_budget: Option<i64>,
    pub cost_estimated: Option<i64>,
    pub pemberi_tugas: Option<String>,
    pub penerima_tugas: Option<String>,
    pub site_document: Option<String>,
    pub stage: Option<String>,
    pub stage_notes: Option<String>,
    pub impl_cico_done: Option<bool>,
    pub impl_rfs_done: Option<bool>,
    pub impl_dokumen_done: Option<bool>,
    pub ineom_registered: Option<bool>,
}

// ==================== STAGE LOG MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteStageLog {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    pub from_stage: String,
    pub to_stage: String,
    pub notes: Option<String>,
    pub changed_by: String,          // user id or name
    pub evidence_urls: Vec<String>,  // file URLs uploaded saat update stage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSiteStageRequest {
    pub stage: String,               // target stage baru
    pub notes: Option<String>,
    pub changed_by: Option<String>,  // user id atau nama yang mengubah
    pub evidence_urls: Option<Vec<String>>,
    pub permit_date: Option<String>, // Tanggal buat permit (wajib saat masuk permit_process)
    pub impl_cico_done: Option<bool>,
    pub impl_rfs_done: Option<bool>,
    pub impl_dokumen_done: Option<bool>,
    pub ineom_registered: Option<bool>,
    // Permit ready stage fields (wajib saat transisi → permit_ready)
    pub tpas_approved: Option<bool>,
    pub tp_approved: Option<bool>,
    pub caf_approved: Option<bool>,
    pub tgl_berlaku_permit_tpas: Option<String>,
    pub tgl_berakhir_permit_tpas: Option<String>,
    // Akses process stage fields (wajib saat transisi → akses_process)
    pub tower_provider: Option<String>,
    pub jenis_kunci: Option<String>,
    pub pic_akses_nama: Option<String>,
    pub pic_akses_telp: Option<String>,
    // Implementasi stage fields (wajib saat transisi → implementasi)
    pub tgl_rencana_implementasi: Option<String>,
    pub tgl_aktual_mulai: Option<String>,
    pub jam_checkin: Option<String>,
    // RFI done stage fields (wajib saat transisi → rfi_done)
    pub jam_checkout: Option<String>,
}

// ==================== TEAM MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    pub nama: String,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub project_id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub leader_id: Option<Thing>,  // FK to people table
    pub active: bool,
    // Employee detail fields (populated from Excel upload)
    pub nik: Option<String>,
    pub nama_karyawan: Option<String>,
    pub tanggal_lahir: Option<String>,
    pub tempat_lahir: Option<String>,
    pub agama: Option<String>,
    pub jenis_kelamin: Option<String>,
    pub no_ktp: Option<String>,
    pub no_hp: Option<String>,
    pub alamat_email: Option<String>,
    pub jabatan_kerja: Option<String>,
    pub regional: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPeople {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub team_id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub people_id: Option<Thing>,
    pub role: Option<String>,
    pub vendor: Option<String>,
    pub device_id: Option<String>,
    pub ime1: Option<String>,
    pub ime2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTeamRequest {
    pub nama: String,
    pub project_id: String,
    pub site_id: Option<String>,
    pub leader_id: Option<String>,
    pub members: Vec<TeamMemberInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTeamRequest {
    pub nama: Option<String>,
    pub project_id: Option<String>,
    pub site_id: Option<String>,
    pub leader_id: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMemberInput {
    pub people_id: String,
    pub role: Option<String>,
    pub vendor: Option<String>,
}

// ==================== SITE TEAM STRUCTURE MODELS ====================
// Tim Struktur: links data master team members to a specific site

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteTeamMember {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub team_master_id: Option<Thing>,  // references teams (master data)
    pub role: Option<String>,
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Enriched Tim Struktur entry with master team member details joined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteTeamMemberDetail {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub team_master_id: Option<Thing>,
    pub role: Option<String>,
    pub vendor: Option<String>,
    // Populated from teams master record
    pub nik: Option<String>,
    pub nama: Option<String>,
    pub no_hp: Option<String>,
    pub jabatan: Option<String>,
    pub regional: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSiteTeamMemberRequest {
    pub team_master_id: String,  // ID of master team record to add
    pub role: Option<String>,
    pub vendor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSiteTeamMemberRequest {
    pub role: Option<String>,
    pub vendor: Option<String>,
}

/// Partial view of master team record used for JOIN enrichment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMasterInfo {
    pub nik: Option<String>,
    pub nama_karyawan: Option<String>,
    pub no_hp: Option<String>,
    pub jabatan_kerja: Option<String>,
    pub regional: Option<String>,
}

// ==================== TEAM UPLOAD RESULT ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamUploadResult {
    pub total_rows: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

// ==================== COST MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub project_id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    pub type_termin: String,
    pub tgl_pengajuan: Option<String>,
    pub jumlah_pengajuan: i64,
    pub tgl_acc: Option<String>,
    pub acc_by: Option<String>,
    pub acc_name: Option<String>,
    pub jumlah_acc: Option<i64>,
    pub tgl_pembayaran: Option<String>,
    pub jumlah_pembayaran: Option<i64>,
    pub status: String,
    pub catatan_tolak: Option<String>,
    pub bukti_transaksi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCostRequest {
    pub project_id: String,
    pub site_id: String,
    pub type_termin: String,
    pub tgl_pengajuan: Option<String>,
    pub jumlah_pengajuan: i64,
    pub status: Option<String>,
    pub catatan_tolak: Option<String>,
}

// ==================== MATERIAL MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    pub skp: Option<String>,
    pub name: String,
    pub unit: String,
    pub qty: i64,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub project_id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    pub tgl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMaterialRequest {
    pub skp: Option<String>,
    pub name: String,
    pub unit: String,
    pub qty: i64,
    pub project_id: String,
    pub site_id: String,
    pub tgl: Option<String>,
}

// ==================== AREA & REGION MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    pub nama_area: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAreaRequest {
    pub nama_area: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub area_id: Option<Thing>,
    pub kode_region: String,
    pub nama_region: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRegionRequest {
    pub area_id: String,
    pub kode_region: String,
    pub nama_region: String,
}

// ==================== FILE MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub project_id: Option<Thing>,
    pub title: String,
    pub filename: String,
    pub original_name: String,
    #[serde(skip_serializing)]  // Hide base64 string from response
    pub file_data: Option<String>,  // Base64 data URL
    pub bucket: Option<String>,
    pub key: String,
    pub mime_type: String,
    pub size: i64,
    pub disk: Option<String>,
    pub visibility: Option<String>,
    pub uploaded_at: Option<String>,
    pub uploaded_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectFileRequest {
    pub project_id: String,
    pub title: String,
    pub filename: String,
    pub original_name: String,
    pub bucket: Option<String>,
    pub key: String,
    pub mime_type: String,
    pub size: i64,
    pub disk: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteFile {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    pub title: String,
    pub filename: String,
    pub original_name: String,
    #[serde(skip_serializing)]  // Hide base64 string from response
    pub file_data: Option<String>,  // Base64 data URL
    pub bucket: Option<String>,
    pub key: String,
    pub mime_type: String,
    pub size: i64,
    pub disk: Option<String>,
    pub visibility: Option<String>,
    pub uploaded_at: Option<String>,
    pub uploaded_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSiteFileRequest {
    pub site_id: String,
    pub title: String,
    pub filename: String,
    pub original_name: String,
    pub bucket: Option<String>,
    pub key: String,
    pub mime_type: String,
    pub size: i64,
    pub disk: Option<String>,
    pub visibility: Option<String>,
}

// ==================== TERMIN MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Termin {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub project_id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    pub type_termin: String,
    pub tgl_terima: Option<String>,
    pub jumlah: i64,
    pub termin_ke: Option<i32>,
    pub percentage: Option<i32>,
    pub status: String,
    pub keterangan: Option<String>,
    
    // Submit tracking
    pub submitted_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    
    // Field Head Review tracking
    pub reviewed_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub catatan_review: Option<String>,
    
    // Director Approval tracking
    pub approved_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub catatan_approval: Option<String>,
    
    // Finance Payment tracking
    pub paid_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub jumlah_dibayar: Option<i64>,
    pub referensi_pembayaran: Option<String>,  // Nomor referensi pembayaran (e.g., TRF-12345B)
    pub catatan_pembayaran: Option<String>,
    #[serde(skip_serializing)]  // Hide base64 string from response (too long & unclear)
    pub bukti_pembayaran: Option<String>,  // Base64 data URL of payment proof
    pub bukti_pembayaran_filename: Option<String>,  // Original filename (e.g., "kwintansi pak adnan.pdf")
    pub bukti_pembayaran_mime_type: Option<String>,  // MIME type (e.g., "application/pdf")
    pub bukti_pembayaran_size: Option<i64>,  // File size in bytes
    // Dokumen pengajuan termin (permit docs, inv proforma, dsb - diupload saat ajukan T)
    #[serde(skip_serializing)]  // Hide base64 string from response (too long)
    pub dokumen_pengajuan: Option<String>,  // Base64 data URL
    pub dokumen_pengajuan_filename: Option<String>,  // Original filename
    pub dokumen_pengajuan_mime_type: Option<String>,  // MIME type
    pub dokumen_pengajuan_size: Option<i64>,  // File size in bytes
    // Rekening tujuan pengajuan termin
    pub nomor_rekening_tujuan: Option<String>,  // e.g., "BCA 1234567890 an. PT Mitra"
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTerminRequest {
    pub project_id: String,
    pub site_id: String,
    pub type_termin: String,
    pub tgl_terima: Option<String>,
    pub jumlah: i64,
    pub termin_ke: i32,
    pub percentage: i32,
    pub status: Option<String>,
    pub keterangan: Option<String>,
    pub submitted_by: Option<String>, // If provided, will submit directly for review
    pub nomor_rekening_tujuan: Option<String>, // Nomor rekening tujuan pengajuan
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTerminRequest {
    pub type_termin: Option<String>,
    pub tgl_terima: Option<String>,
    pub jumlah: Option<i64>,
    pub keterangan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTerminRequest {
    pub submitter_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewTerminRequest {
    pub reviewer_name: String,
    pub catatan_review: String,
    pub approve: bool, // true = approve, false = reject
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveTerminRequest {
    pub approver_name: String,
    pub catatan_approval: Option<String>,
    pub approve: bool, // true = approve, false = reject
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayTerminRequest {
    pub payer_name: String,
    pub jumlah_dibayar: i64,
    pub referensi_pembayaran: String,  // Required: Nomor referensi pembayaran
    pub catatan_pembayaran: Option<String>,
    pub bukti_pembayaran: Option<String>,
}

// ==================== TERMIN WITH SITE INFO MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminSiteInfo {
    pub site_name: String,
    pub project_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminWithSiteInfo {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub project_id: Option<Thing>,
    pub site_id: Option<TerminSiteInfo>,
    pub type_termin: String,
    pub tgl_terima: Option<String>,
    pub jumlah: i64,
    pub termin_ke: Option<i32>,
    pub percentage: Option<i32>,
    pub status: String,
    pub keterangan: Option<String>,
    
    // Submit tracking
    pub submitted_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    
    // Field Head Review tracking
    pub reviewed_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub catatan_review: Option<String>,
    
    // Director Approval tracking
    pub approved_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub catatan_approval: Option<String>,
    
    // Finance Payment tracking
    pub paid_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub jumlah_dibayar: Option<i64>,
    pub referensi_pembayaran: Option<String>,
    pub catatan_pembayaran: Option<String>,
    #[serde(skip_serializing)]  // Hide base64 string from response (too long & unclear)
    pub bukti_pembayaran: Option<String>,
    pub bukti_pembayaran_filename: Option<String>,
    pub bukti_pembayaran_mime_type: Option<String>,
    pub bukti_pembayaran_size: Option<i64>,
    pub nomor_rekening_tujuan: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ==================== TERMIN FILE MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminFile {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub termin_id: Option<Thing>,
    pub category: Option<String>,
    pub title: String,
    pub filename: String,
    pub original_name: String,
    #[serde(skip_serializing)]  // Hide base64 string from response
    pub file_data: Option<String>,  // Base64 data URL
    pub bucket: Option<String>,
    pub key: String,
    pub mime_type: String,
    pub size: i64,
    pub disk: Option<String>,
    pub visibility: Option<String>,
    pub uploaded_at: Option<String>,
    pub uploaded_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTerminFileRequest {
    pub termin_id: String,
    pub category: Option<String>,
    pub title: String,
    pub filename: String,
    pub original_name: String,
    pub bucket: Option<String>,
    pub key: String,
    pub mime_type: String,
    pub size: i64,
    pub disk: Option<String>,
    pub visibility: Option<String>,
}

// ==================== USER MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    pub name: String,
    pub email: String,
    pub role: String,
    pub email_verified_at: Option<String>,
    #[serde(skip_serializing)]
    pub password: String,
    pub remember_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub password: Option<String>,
}

// ==================== BULK IMPORT MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportExcelResponse {
    pub project: Project,
    pub total_rows: usize,
    pub sites_created: usize,
    pub sites_failed: usize,
    pub created_sites: Vec<Site>,
    pub errors: Vec<ImportError>,
    pub summary: ImportSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportError {
    pub row_number: usize,
    pub field: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSummary {
    pub project_id: String,
    pub project_name: String,
    pub total_budget: i64,
    pub sites_count: usize,
    pub message: String,
}

// ==================== SITE BOQ MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteBoq {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    pub item_code: String,
    pub description: String,
    pub quantity: f64,
    pub unit: String,
    #[serde(rename = "type")]
    pub boq_type: Option<String>,   // 'material' | 'jasa'
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSiteBoqRequest {
    pub item_code: String,
    pub description: String,
    pub quantity: f64,
    pub unit: String,
    #[serde(rename = "type")]
    pub boq_type: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSiteBoqRequest {
    pub item_code: Option<String>,
    pub description: Option<String>,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    #[serde(rename = "type")]
    pub boq_type: Option<String>,
    pub source: Option<String>,
}

// ==================== SKP MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skp {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    pub skp_number: String,
    pub tanggal: String,
    pub keterangan: Option<String>,
    pub status: Option<String>,         // Draft | Submitted | Received
    pub uploaded_by: String,
    pub document_url: Option<String>,
    pub received_proof_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSkpRequest {
    pub skp_number: String,
    pub tanggal: String,
    pub keterangan: Option<String>,
    pub uploaded_by: String,
    pub document_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSkpRequest {
    pub skp_number: Option<String>,
    pub tanggal: Option<String>,
    pub keterangan: Option<String>,
    pub status: Option<String>,         // Draft | Submitted | Received
    pub document_url: Option<String>,
    pub received_proof_url: Option<String>,
}

// ==================== SITE PERMIT DOCUMENT MODELS ====================

/// Dokumen perizinan yang diupload saat transisi ke permit_ready.
/// Disimpan terpisah dari site_evidence agar mudah diakses per site/stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitePermitDoc {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    pub filename: String,
    pub original_name: Option<String>,
    pub file_url: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    /// Jenis dokumen: "tpas" | "tp" | "caf" | "lainnya"
    pub doc_type: String,
    pub uploaded_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploaded_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ==================== SITE EVIDENCE MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteEvidence {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    pub filename: String,
    pub original_name: Option<String>,
    pub file_url: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub progress_tag: String,
    pub stage_context: Option<String>,
    pub uploaded_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploaded_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSiteEvidenceRequest {
    pub filename: String,
    pub original_name: Option<String>,
    pub file_url: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub progress_tag: String,
    pub stage_context: Option<String>,
    pub uploaded_by: String,
}

// ==================== SITE ISSUE MODELS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteIssue {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub id: Option<Thing>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "thing_serializer::serialize")]
    pub site_id: Option<Thing>,
    /// Stage saat issue dilaporkan
    pub stage_at_report: String,
    pub keterangan: String,
    /// 'tahan' | 'eskalasi'
    pub tindakan: String,
    /// 'open' | 'resolved' | 'escalated'
    pub status: Option<String>,
    pub reported_by: Option<String>,
    pub evidence_urls: Option<Vec<String>>,
    pub resolved_by: Option<String>,
    pub resolved_notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSiteIssueRequest {
    pub stage_at_report: String,
    pub keterangan: String,
    /// 'tahan' | 'eskalasi'
    pub tindakan: String,
    pub reported_by: Option<String>,
    pub evidence_urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveSiteIssueRequest {
    pub resolved_by: String,
    pub resolved_notes: Option<String>,
}

// ==================== RESPONSE WRAPPER ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}
