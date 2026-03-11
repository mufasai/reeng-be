# 📚 Reengineering Tool Backend - API Documentation

**Base URL:** `http://localhost:3001/api`

---

## 📋 Changelog

### v2.1.0 (2026-03-11)
**🚨 Site Issue / Blocker Management**
- ✅ **NEW ENDPOINT:** `POST /sites/:id/issues` — Laporkan issue/blocker di stage saat ini
  - Tindakan `tahan`: hold di stage, status `open`
  - Tindakan `eskalasi`: eskalasi ke management, status `escalated`
- ✅ **NEW ENDPOINT:** `GET /sites/:id/issues` — List semua issue per site
- ✅ **NEW ENDPOINT:** `GET /site-issues/:issue_id` — Detail satu issue
- ✅ **NEW ENDPOINT:** `POST /site-issues/:issue_id/resolve` — Resolve/selesaikan issue (status → `resolved`)
- ✅ **NEW ENDPOINT:** `DELETE /site-issues/:issue_id` — Hapus issue
- ✅ **NEW ENDPOINT:** `GET /api/sites/:id` — Get detail site by ID
- 🗄️ **NEW TABLE:** `site_issue` — menyimpan keterangan, tindakan, status, evidence, resolved info

### v2.0.0 (2026-03-10)
**📍 Site Progress Tracking — Stage, BOQ, SKP & Evidence**
- ✅ **NEW ENDPOINT:** `POST /sites/:id/stage` — Update stage/progress site + catat audit log otomatis
  - 13 stage valid: `imported → assigned → permit_process → permit_ready → akses_process → akses_ready → implementasi → rfi_done → rfs_done → dokumen_done → bast → invoice → completed`
  - Menyimpan `from_stage`, `to_stage`, `changed_by`, `notes`, `evidence_urls` di `site_stage_log`
  - Field tambahan: `impl_cico_done`, `impl_rfs_done`, `impl_dokumen_done`, `ineom_registered`
- ✅ **NEW ENDPOINT:** `GET /sites/:id/stage-logs` — Riwayat perubahan stage (audit trail)
- ✅ **NEW ENDPOINT:** `GET /sites/:site_id/boq` — List Bill of Quantity material per site
- ✅ **NEW ENDPOINT:** `POST /sites/:site_id/boq` — Tambah item BOQ
- ✅ **NEW ENDPOINT:** `PUT /site-boq/:boq_id` — Update item BOQ
- ✅ **NEW ENDPOINT:** `DELETE /site-boq/:boq_id` — Hapus item BOQ
- ✅ **NEW ENDPOINT:** `GET /sites/:site_id/skp` — List SKP (Surat Perintah Ambil Material) per site
- ✅ **NEW ENDPOINT:** `POST /sites/:site_id/skp` — Buat SKP baru
- ✅ **NEW ENDPOINT:** `GET /skp/:skp_id` — Detail satu SKP
- ✅ **NEW ENDPOINT:** `PUT /skp/:skp_id` — Update SKP (termasuk status: Draft→Submitted→Received)
- ✅ **NEW ENDPOINT:** `DELETE /skp/:skp_id` — Hapus SKP
- ✅ **NEW ENDPOINT:** `GET /sites/:site_id/evidence` — List foto lapangan per site
- ✅ **NEW ENDPOINT:** `POST /sites/:site_id/evidence` — Upload metadata foto lapangan
- ✅ **NEW ENDPOINT:** `DELETE /site-evidence/:evidence_id` — Hapus foto lapangan
- 🗄️ **NEW TABLES:** `site_stage_log`, `site_boq`, `skp`, `site_evidence` — semua terverifikasi di SurrealDB
- 📚 **New Models:** `SiteBoq`, `Skp`, `SiteEvidence` + semua request/response struct
- 🏗️ **Stage Fields on `sites` table:** `stage` (DEFAULT 'imported'), `stage_updated_at`, `stage_notes`, `impl_cico_done`, `impl_rfs_done`, `impl_dokumen_done`, `ineom_registered`

### v1.9.0 (2026-03-06)
**👥 Tim Struktur - Site Team Structure Management**
- ✅ **NEW ENDPOINT:** `GET /sites/:site_id/team-structure` — List Tim Struktur per site
  - Response enriched dengan data master: `nik`, `nama`, `no_hp`, `jabatan`, `regional`
- ✅ **NEW ENDPOINT:** `POST /sites/:site_id/team-structure` — Add member dari Data Master Team
  - Pick member dari `GET /teams` (data master), klik → otomatis masuk Tim Struktur
  - Duplicate prevention: member yang sama tidak bisa ditambah 2x ke site yang sama
  - Body: `{ "team_master_id": "teams:xxx", "role": "member", "vendor": "Vendor A" }`
- ✅ **NEW ENDPOINT:** `DELETE /sites/:site_id/team-structure/:member_id` — Remove member dari Tim Struktur (tidak menghapus data master)
- ✅ **FIXED:** `GET /teams` sekarang menampilkan **semua** data master team (sebelumnya ada filter 1 hari yang salah)
- ✅ **CASCADE DELETE:** Saat site dihapus, semua `site_team_members` ikut terhapus otomatis
- 🏗️ **NEW TABLE:** `site_team_members` dengan field: `site_id`, `team_master_id`, `role`, `vendor`
- 📚 **New Models:** `SiteTeamMember`, `SiteTeamMemberDetail`, `AddSiteTeamMemberRequest`, `TeamMasterInfo`
- 🎯 **Tested Live:** Semua endpoint telah diuji dan verified di backend running

### v1.8.0 (2026-03-05)
**📊 Excel Bulk Import - Projects & Sites Creation**
- ✅ **NEW ENDPOINT:** `POST /projects/import-excel` - Bulk import from Excel
- ✅ **MULTIPART UPLOAD:** Accept .xlsx files via multipart/form-data
- ✅ **SMART PARSING:** Auto-extract project info from filename and Row 2 totals
- ✅ **SHEET 3 SUPPORT:** Parse "Active Project Details" sheet specifically
- ✅ **COLUMN MAPPING:** 15+ Excel columns mapped to Site model fields
- ✅ **ATOMIC CREATION:** Create 1 Project + 100+ Sites in single request
- ✅ **ERROR HANDLING:** Continue-on-error with detailed per-row error reporting
- ✅ **AUTO-GENERATE:** Missing nomor_kontrak auto-generated with timestamp
- ✅ **DATE FLEXIBILITY:** Support Excel datetime, YYYY-MM-DD, DD/MM/YYYY formats
- ✅ **RELATIONAL:** All sites automatically linked to created project
- 🏷️ **TYPE AUTO-DETECT:** ⭐ NEW - Project type & name from Column B (TIPE PROJECT)
  - Supported types: COMBAT, L2H, BLACK SITE, REFINEN, FILTER, BEBAN OPERASIONAL
  - Project name format: `{TIPE} Project {LOKASI}` (e.g., "FILTER Project Jakarta")
  - Column B Row 6 determines project type for all sites
- 📦 **Dependencies:** Added `calamine = { version = "0.24", features = ["dates"] }` for Excel parsing
- 📝 **Response:** Complete import summary with project, sites array, error list, statistics
- 🎯 **Impact:** Drastically reduce data entry time - import 100+ sites in seconds vs hours of manual entry
- 💡 **Use Case:** Bulk onboarding of OSP/Filter/Combat projects from Telkom Excel reports
- 🔧 **New Handler:** `src/handlers/bulk_import.rs` (450+ lines with type detection)
- 📚 **Models:** `BulkImportExcelResponse`, `ImportError`, `ImportSummary`

### v1.7.1 (2026-03-04)
**📌 Termin Response Enhancement - Project Name Display**
- ✅ **ENHANCED RESPONSE:** Termin list endpoints sekarang include `project_name`
  - Field `site_id` dalam response berisi object: `{ site_name: string, project_name: string }`
  - Berlaku untuk: `GET /termins`, `GET /termins/project/:id`, `GET /termins/site/:id`
- ✅ **BACKEND OPTIMIZATION:** Auto-fetch project name dari database (no frontend query needed)
- ✅ **MODEL UPDATE:** `TerminSiteInfo` struct ditambahkan field `project_name`
- ✅ **NEW ENDPOINT:** `GET /teams/leader/:leader_id` - Get team by leader ID
  - Query team berdasarkan leader_id (person yang jadi leader)
  - Useful untuk cek team mana yang dipimpin oleh seseorang
- 🎯 **Impact:** Frontend bisa langsung tampilkan project name di menu termin (1-4) tanpa query tambahan
- 📊 **Use Case:** User dapat melihat "Project → Site → Termin" hierarchy dengan jelas
- 🔧 **Updated Handlers:** `list_termins`, `get_termins_by_project`, `get_termins_by_site`, `get_team_by_leader`

### v1.7.0 (2026-03-03)
**👥 Teams CRUD & 📁 Multipart File Uploads (Project, Site, Termin)**
- ✅ **NEW MODULE:** Teams Management - Full CRUD operations
  - `POST /teams` - Create team dengan members
  - `GET /teams` - List all teams
  - `GET /teams/:team_id` - Get team detail
  - `PUT /teams/:team_id` - Update team info
  - `DELETE /teams/:team_id` - Delete team (cascade delete members)
  - `GET /teams/:team_id/members` - List team members
- ✅ **NEW ROLE:** `head office` - Role untuk Head Office user
- ✅ **MULTIPART FILE UPLOADS:** Project, Site, dan Termin sekarang support upload file real dengan base64 storage
  - `POST /projects/:id/upload` - Upload file ke project (multipart)
  - `POST /sites/:id/upload` - Upload file ke site (multipart)
  - `POST /termins/:id/upload` - Upload file ke termin (multipart)
- ✅ **DOWNLOAD ENDPOINTS:** Download file yang sudah diupload
  - `GET /project-files/:file_id/download` - Download project file
  - `GET /site-files/:file_id/download` - Download site file
  - `GET /termin-files/:file_id/download` - Download termin file
- ✅ **FILE STORAGE:** File disimpan sebagai base64 data URL di field `file_data` (hidden dari response)
- ✅ **CLEAN MODELS:** Field `file_data` menggunakan `#[serde(skip_serializing)]` untuk response yang bersih
- 🎯 **Impact:** Complete file management system untuk Project, Site, Termin dengan download support
- 📦 **Storage:** Base64 storage di database, tidak perlu S3 atau disk storage
- 👥 **Teams:** Manage tim dengan leader, members, vendor info, device tracking

### v1.6.0 (2026-03-01)
**📥 Unified Payment Endpoint & File Download**
- ✅ **UNIFIED ENDPOINT:** `/termins/:id/pay` sekarang mendukung 2 content types:
  - `application/json` - Pembayaran tanpa file
  - `multipart/form-data` - Pembayaran dengan file upload
- ✅ **AUTO-DETECT:** Backend otomatis detect Content-Type dan proses sesuai format
- ✅ **FILE METADATA:** Menyimpan filename, mime_type, dan size terpisah
- ✅ **CLEAN RESPONSE:** Field base64 di-hide dari JSON response (hanya metadata yang muncul)
- ✅ **NEW ENDPOINT:** `GET /termins/:id/download-bukti-pembayaran` untuk download file
- ✅ **DOWNLOAD SUPPORT:** File download dengan nama asli dan mime type correct
- 🎯 **Impact:** Single endpoint untuk semua payment scenario, user tidak bingung
- 📥 **Usage:** Di Postman gunakan "Send and Download" untuk test download endpoint
- 🗄️ **Storage:** Base64 tetap tersimpan di database untuk download, tapi hidden dari response

### v1.5.0 (2026-02-28)
**🔄 File Storage Optimization & Role Addition**
- ✅ **BREAKING CHANGE:** File storage dipisahkan ke tabel terpisah `bukti_pembayaran_files`
- ✅ Field `bukti_pembayaran` dihapus dari model `Termin` (hanya metadata yang tersisa)
- ✅ Tabel `termins` sekarang hanya menyimpan metadata: `bukti_pembayaran_filename`, `bukti_pembayaran_mime_type`, `bukti_pembayaran_size`
- ✅ Base64 content disimpan di tabel `bukti_pembayaran_files` dengan referensi `termin_id`
- ✅ Endpoint download tetap sama: `/termins/:id/bukti-pembayaran`
- ✅ Tambah endpoint cleanup: `POST /termins/cleanup/old-bukti-pembayaran` untuk migrasi data lama
- ✅ **NEW ROLE:** Tambah role `direktur` ke sistem user management
- 🎯 **Impact:** Database termins lebih bersih di Surrealist Explorer, tidak ada string base64 panjang
- 📊 **Benefits:** Improved query performance, cleaner table view, better data organization
- 👥 **Roles:** 7 role tersedia: backoffice admin, management, team leader, finance, engineer, admin, **direktur**

### v1.4.1 (2026-02-27)
**📎 Payment File Upload - Direct Database Storage**
- ✅ **BREAKING CHANGE:** Endpoint `/termins/:id/pay` sekarang menggunakan **multipart/form-data**
- ✅ Field `bukti_pembayaran` sekarang menerima **file upload langsung**
- ✅ **File disimpan langsung ke database SurrealDB sebagai base64** (BUKAN ke disk!)
- ✅ Tambah field metadata: `bukti_pembayaran_filename`, `bukti_pembayaran_mime_type`, `bukti_pembayaran_size`
- ✅ Response berisi base64 encoded file content yang bisa di-decode kembali
- ✅ Support berbagai format: PDF, JPG, PNG, TXT, dll
- 🎯 **Impact:** File bukti pembayaran tersimpan aman di database, tidak perlu storage eksternal
- ⚠️ **Migration Note:** Frontend perlu update dari JSON request ke multipart form-data

### v1.4.0 (2026-02-27)
**👥 User Management & Registration System**
- ✅ Implementasi endpoint **Register** dengan pilihan role
- ✅ Implementasi **User Management** CRUD (Create, Read, Update, Delete)
- ✅ Update model User dengan field `role` (required)
- ✅ 6 role yang tersedia: backoffice admin, management, team leader, finance, engineer, admin
- ✅ Validasi email uniqueness saat register
- ✅ Password hashing untuk keamanan
- ✅ Update Login untuk menggunakan database users
- 🎯 **Impact:** Sistem sekarang mendukung multi-user dengan role-based access

### v1.3.1 (2026-02-26)
**💡 Termin Flexible Amount - Design Optimization**
- ✅ **Jumlah Flexible:** Field `jumlah` tidak lagi wajib match dengan `percentage × maximal_budget`
- ✅ **Percentage as Label:** Field `percentage` tetap terkunci (30-50-10-10) sebagai struktur termin
- ✅ **70% Compliance:** Memungkinkan semua 4 termin dibuat dalam limit 70% dengan adjust jumlah
- 🎯 **Rationale:** Menyelesaikan konflik antara pattern 30-50-10-10 (=100%) dengan limit 70%
- 📝 **Example:** Termin 2 (50%) bisa diisi jumlah 20 juta (tidak harus 50 juta dari budget 100 juta)
- 🔧 **Changed:** Removed exact amount validation, kept percentage pattern + sequential + 70% limit

### v1.3.0 (2026-02-26)
**🔒 Termin Business Rules Enforcement - Critical Validations**
- ✅ **VALIDASI 1:** Enforcement pola percentage terkunci (30%-50%-10%-10%)
  - Termin 1 WAJIB 30%, Termin 2 WAJIB 50%, Termin 3 WAJIB 10%, Termin 4 WAJIB 10%
  - Request dengan percentage berbeda akan ditolak dengan error
- ✅ **VALIDASI 2:** Sequential termin dependency check
  - Termin hanya bisa dibuat setelah termin sebelumnya berstatus "approved" atau "paid"
  - Mencegah pembuatan termin secara paralel atau melompat urutan
- ✅ **VALIDASI 3:** Maximum payment limit 70% dari site value
  - Total kumulatif semua termin tidak boleh melebihi 70% dari `maximal_budget`
  - Sistem menghitung jumlah total termin existing + termin baru
  - Mencegah overpayment di level site
- ✅ Field `termin_ke` dan `percentage` sekarang **REQUIRED** (tidak lagi optional)
- 🎯 **Impact:** Sistem sekarang fully compliant dengan business rules untuk mencegah kebocoran budget

### v1.2.1 (2026-02-26)
**Termin Payment - Add Payment Reference Field**
- ✅ Tambah field `referensi_pembayaran` (required) di model Termin
- ✅ Tambah field `referensi_pembayaran` (required) di PayTerminRequest
- ✅ Update endpoint POST `/termins/:id/pay` untuk menyimpan nomor referensi
- 📝 Field ini untuk tracking nomor referensi pembayaran (e.g., TRF-12345B, INV-001)

### v1.2.0 (2026-02-26)
**Termin API - Backward Compatibility Update**
- ✅ Field `termin_ke` dan `percentage` sekarang **optional** (sebelumnya required)
- ✅ Mendukung data termin lama yang belum memiliki field tersebut
- ✅ Validasi otomatis hanya berjalan jika `percentage` field diisi
- ⚠️ **Rekomendasi**: Untuk termin baru, tetap isi `termin_ke` dan `percentage` untuk aktivasi validasi otomatis

### v1.1.0 (2026-02-25)
**Termin Workflow & Validation**
- ✅ Implementasi full workflow: draft → pending_review → reviewed → approved → paid
- ✅ Tambah field `termin_ke` dan `percentage` untuk tracking urutan termin
- ✅ Validasi otomatis: `jumlah = percentage × site.maximal_budget`
- ✅ Direct submit feature: Optional `submitted_by` untuk langsung submit saat create
- ✅ 15 endpoints termin lengkap (CRUD + workflow + files)

### v1.0.0 (2026-02-20)
**Initial Release**
- ✅ Core APIs: Projects, Sites, People, Teams, Costs, Materials
- ✅ Authentication dengan JWT
- ✅ File management untuk project & site
- ✅ Regional management (Areas & Regions)

---

## 🔐 Authentication & User Management

### Register
**POST** `/auth/register`

**Request Body:**
```json
{
  "name": "John Doe",
  "email": "john.doe@smartelco.com",
  "password": "securepassword123",
  "role": "engineer"
}
```

**Available Roles:**
- `backoffice admin`
- `management`
- `team leader`
- `finance`
- `engineer`
- `admin`
- `direktur`

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "users:abc123def456",
    "name": "John Doe",
    "email": "john.doe@smartelco.com",
    "role": "engineer"
  },
  "message": "User registered successfully"
}
```

**Response (Error - Email Already Exists):**
```json
{
  "success": false,
  "data": null,
  "message": "Email already registered"
}
```

**Response (Error - Invalid Email):**
```json
{
  "success": false,
  "data": null,
  "message": "Invalid email format"
}
```

---

### Login
**POST** `/auth/login`

**Request Body:**
```json
{
  "email": "john.doe@smartelco.com",
  "password": "securepassword123"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "token": "token_john.doe@smartelco.com_1709049600",
  "user": {
    "id": "users:abc123def456",
    "name": "John Doe",
    "email": "john.doe@smartelco.com",
    "role": "engineer"
  },
  "message": "Login successful"
}
```

**Response (401 Unauthorized):**
```json
{
  "success": false,
  "token": null,
  "user": null,
  "message": "Invalid credentials"
}
```

---

### Get All Users
**GET** `/users`

**Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "users:abc123def456",
      "name": "John Doe",
      "email": "john.doe@smartelco.com",
      "role": "engineer",
      "email_verified_at": null,
      "remember_token": null,
      "created_at": "2026-02-27T10:30:00Z",
      "updated_at": "2026-02-27T10:30:00Z"
    },
    {
      "id": "users:xyz789ghi012",
      "name": "Jane Smith",
      "email": "jane.smith@smartelco.com",
      "role": "finance",
      "email_verified_at": null,
      "remember_token": null,
      "created_at": "2026-02-27T11:00:00Z",
      "updated_at": "2026-02-27T11:00:00Z"
    }
  ],
  "message": null
}
```

---

### Get User By ID
**GET** `/users/:user_id`

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "users:abc123def456",
    "name": "John Doe",
    "email": "john.doe@smartelco.com",
    "role": "engineer",
    "email_verified_at": null,
    "remember_token": null,
    "created_at": "2026-02-27T10:30:00Z",
    "updated_at": "2026-02-27T10:30:00Z"
  },
  "message": null
}
```

**Response (404 Not Found):**
```json
{
  "success": false,
  "data": null,
  "message": "User not found"
}
```

---

### Update User
**PUT** `/users/:user_id`

**Request Body (all fields optional):**
```json
{
  "name": "John Doe Updated",
  "email": "john.updated@smartelco.com",
  "role": "team leader",
  "password": "newpassword123"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "users:abc123def456",
    "name": "John Doe Updated",
    "email": "john.updated@smartelco.com",
    "role": "team leader",
    "email_verified_at": null,
    "remember_token": null,
    "created_at": "2026-02-27T10:30:00Z",
    "updated_at": "2026-02-27T14:25:00Z"
  },
  "message": "User updated successfully"
}
```

**Response (Error - No Fields):**
```json
{
  "success": false,
  "data": null,
  "message": "No fields to update"
}
```

---

### Delete User
**DELETE** `/users/:user_id`

**Response (200 OK):**
```json
{
  "success": true,
  "data": null,
  "message": "User deleted successfully"
}
```

---

## 📁 Projects API

### Create Project
**POST** `/projects`

**Request Body:**
```json
{
  "name": "Network Expansion Jakarta",
  "lokasi": "Jakarta",
  "value": 5000000000,
  "cost": 0,
  "tipe": "COMBAT",
  "keterangan": "Ekspansi jaringan fiber optik Jakarta 2026",
  "tgi_start": "2026-03-01",
  "tgi_end": "2026-12-31",
  "status": "active"
}
```

**Field Definitions:**
- `name` (string, required): Nama project
- `lokasi` (string, required): Lokasi project
- `value` (integer, required): Nilai/anggaran project (dalam Rupiah)
- `cost` (integer, optional): Biaya yang sudah dikeluarkan (default: 0)
- `tipe` (enum, required): Tipe project
  - `"COMBAT"`
  - `"L2H"`
  - `"BLACK SITE"`
  - `"REFINEN"`
  - `"FILTER"`
  - `"BEBAN OPERASIONAL"`
- `keterangan` (string, required): Deskripsi/keterangan project
- `tgi_start` (string, optional): Tanggal mulai (format: YYYY-MM-DD)
- `tgi_end` (string, optional): Tanggal selesai (format: YYYY-MM-DD)
- `status` (string, optional): Status project (default: "active")

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "projects:b7v5e43bvtpwyipxlemg",
    "name": "Network Expansion Jakarta",
    "lokasi": "Jakarta",
    "value": 5000000000,
    "cost": 0,
    "keterangan": "Ekspansi jaringan fiber optik Jakarta 2026",
    "tipe": "COMBAT",
    "tgi_start": "2026-03-01",
    "tgi_end": "2026-12-31",
    "status": "active",
    "created_at": "2026-02-20T09:00:00Z",
    "updated_at": "2026-02-20T09:00:00Z"
  },
  "message": "Project created successfully"
}
```

### List All Projects
**GET** `/projects`

**Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "projects:b7v5e43bvtpwyipxlemg",
      "name": "Network Expansion Jakarta",
      "lokasi": "Jakarta",
      "value": 5000000000,
      "cost": 0,
      "tipe": "COMBAT",
      "keterangan": "Ekspansi jaringan...",
      "tgi_start": "2026-03-01",
      "tgi_end": "2026-12-31",
      "status": "active",
      "created_at": "2026-02-20T09:00:00Z",
      "updated_at": "2026-02-20T09:00:00Z"
    }
  ],
  "message": null
}
```

### Bulk Import from Excel
**POST** `/projects/import-excel`

**Content-Type:** `multipart/form-data`

**Request (Multipart Form):**
- **Field name:** `file`
- **Type:** File upload
- **Accepted formats:** `.xlsx` (Excel 2007+)
- **Max size:** 10MB (recommended)

**Excel File Structure Requirements:**

1. **Sheet:** Must have sheet named **"Active Project Details"** (Sheet 3)

2. **Filename Format (Flexible):**
   - **EPROC Format:** `EPROC{DATE}_{Company}_{Type}_{Category}_{Batch}_{Location}.xlsx`
     - Example: `EPROC20251209002_Smartelco_BoQ_Assignment_Filter_Batch_2_Jabo.xlsx`
     - Project name: `FILTER Project Jabo`
   - **SST Format:** `EPROC{DATE}_SST_{Type}_{Category}_{Details}_{Location}.xlsx`
     - Example: `EPROC20260206001_SST_BOQ_IRR_Filter_Batch_5_and_4_R12_Eastern_Jakarta.xlsx`
     - Project name: `FILTER Project Jakarta`
   - **OSP Format:** `OSP Project Report_Update-YYYYMMDD-LOCATION.xlsx`
     - Example: `OSP Project Report_Update-20260215-PEKALONGAN.xlsx`
     - Project name: `{TIPE} Project PEKALONGAN` (TIPE from Column B)
   - **Auto-extraction:**
     - Location: Last part of filename (e.g., "Jabo", "Jakarta", "PEKALONGAN")
     - Date: First 8-digit number in filename (YYYYMMDD format)

3. **Excel Layout:**
   - **Row 2:** Summary totals
     - Column I (index 8): BOQ AKTUAL → Project `value`
     - Column M (index 12): TOTAL NILAI PO → Project `cost`
   - **Row 5 (index 4):** Column headers
   - **Row 6 (index 5):** First data row - **Column B contains TIPE PROJECT**
   - **Row 6+ (index 5+):** Site data rows (all sites will use same TIPE)

4. **Column Mapping (0-indexed):**
   - **Column B (1):** TIPE PROJECT → `project.tipe` & `project.name` ⭐ **NEW**
     - Supported types: COMBAT, L2H, BLACK SITE, REFINEN, FILTER, BEBAN OPERASIONAL
     - Project name format: `{TIPE} Project {LOKASI}`
     - Example: "FILTER Project Jakarta", "COMBAT Project Surabaya"
   - **Column L (11):** NAMA LOP [SITE] → `site_name` *(required)*
   - **Column D (3):** WTIEL → `lokasi`
   - **Column K (10):** NAMA PO → `pekerjaan`
   - **Column J (9):** NOMOR PO → `nomor_kontrak`
   - **Column G (6):** TANGGAL WO → `start` (date)
   - **Column O (14):** TANGGAL → `end` (date, fallback to start)
   - **Column M (12):** NILAI PO → `maximal_budget` (integer)
   - **Column H (7):** BOQ KONTRAK → `cost_estimated` (integer)
   - **Column B+N+P (1,13,15):** Combined → `site_info` (includes TIPE, STATUS, KETERANGAN)

5. **Auto-generated Fields:**
   - `pemberi_tugas`: "PT Telkom Indonesia"
   - `penerima_tugas`: "Vendor/Pelaksana"
   - `nomor_kontrak`: Auto-generated if empty (`PO-{row}-{timestamp}`)
   - `latitude`, `longitude`: null (can be updated later)
   - `site_document`: null

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "project": {
      "id": "projects:cyhxgkwerejwfv3rb61a",
      "name": "FILTER Project Jakarta",
      "lokasi": "Jakarta",
      "value": 257091760,
      "cost": 87204220,
      "keterangan": "Progress Projek FILTER - Import from Excel: EPROC20260206001_SST_BOQ_IRR_Filter_Batch_5_and_4_R12_Eastern_Jakarta.xlsx",
      "tipe": "FILTER",
      "tgi_start": "2026-02-06",
      "tgi_end": null,
      "status": "active",
      "created_at": "2026-03-05T02:11:42.123456Z",
      "updated_at": "2026-03-05T02:11:42.123456Z"
    },
    "total_rows": 36,
    "sites_created": 36,
    "sites_failed": 0,
    "created_sites": [
      {
        "id": "sites:smks0uk6zupih39jzsf2",
        "project_id": "projects:cyhxgkwerejwfv3rb61a",
        "site_name": "PT3-24-BLU-FY-JAWA TENGAH_5330_add",
        "site_info": "PT3_PT4_SMG | Status: 8. REKONSILIASI MATERIAL | done BACT, ogp pelurusan matrial",
        "pekerjaan": "OSP FO LOKASI BLU-FY-JawaTengah_5330_add WITEL PEKALONGAN",
        "lokasi": "PEKALONGAN",
        "latitude": null,
        "longitude": null,
        "nomor_kontrak": "4200032602",
        "start": "2024-10-18",
        "end": "2026-03-05",
        "maximal_budget": 0,
        "cost_estimated": 8020646,
        "pemberi_tugas": "PT Telkom Indonesia",
        "penerima_tugas": "Vendor/Pelaksana",
        "site_document": null,
        "created_at": "2026-03-05T02:11:42.913959Z",
        "updated_at": "2026-03-05T02:11:42.913960Z"
      }
      // ... 35 more sites
    ],
    "errors": [],
    "summary": {
      "project_id": "projects:cyhxgkwerejwfv3rb61a",
      "project_name": "OSP Project PEKALONGAN",
      "total_budget": 257091760,
      "sites_count": 36,
      "message": "Import completed: 36 sites created, 0 failed out of 36 rows"
    }
  },
  "message": null
}
```

**Error Response Fields:**
```json
{
  "success": true,
  "data": {
    "errors": [
      {
        "row_number": 15,
        "field": "site_name",
        "message": "Site name (Column L) is required but empty",
        "data": null
      },
      {
        "row_number": 22,
        "field": "database",
        "message": "Failed to create site: database error",
        "data": {
          "site_name": "Test Site"
        }
      }
    ]
  }
}
```

**Error Scenarios:**
- **400 Bad Request:** No file uploaded, or invalid filename
- **500 Internal Server Error:** Excel parsing failed, database error, or invalid sheet structure

**Notes:**
- ✅ **Atomic:** Creates 1 Project + N Sites in single operation
- ✅ **Resilient:** Continues processing even if some rows fail
- ✅ **Informative:** Returns detailed error per row
- ✅ **Flexible:** Skips empty rows automatically
- ✅ **Relational:** All sites linked to created project via `project_id`
- ⚠️ **Date Parsing:** Supports Excel datetime, "YYYY-MM-DD", "DD/MM/YYYY" formats
- 💡 **Tip:** Use Postman's file upload feature to test with actual Excel file

**cURL Example:**
```bash
curl -X POST http://localhost:3000/api/projects/import-excel \
  -F 'file=@/path/to/OSP Project Report_Update-20260215-PEKALONGAN.xlsx'
```

**Postman Setup:**
1. Create new POST request
2. URL: `{{base_url}}/projects/import-excel`
3. Body → `form-data`
4. Add key `file` with type `File`
5. Select your Excel file
6. Send request

---

## 🏗️ Sites API

### Create Site (with Team Members)
**POST** `/sites`

**Request Body:**
```json
{
  "project_id": "projects:b7v5e43bvtpwyipxlemg",
  "site_name": "Site Menteng",
  "site_info": "Area Menteng Jakarta Pusat dengan 500 rumah",
  "pekerjaan": "Instalasi Fiber to Home",
  "lokasi": "Menteng, Jakarta Pusat",
  "nomor_kontrak": "KTR/2026/001",
  "start": "2026-03-15",
  "end": "2026-07-15",
  "maximal_budget": 500000000,
  "cost_estimated": 450000000,
  "pemberi_tugas": "PT Telkom Indonesia",
  "penerima_tugas": "PT SmartElco Solutions",
  "site_document": null,
  "team_members": [
    "people:1q9t3fd5jiu07j1jl2jj",
    "people:3h6pq9fhkkmshx7tyksz"
  ]
}
```

**Field Definitions:**
- `project_id` (string, required): ID project (format: "projects:xxx")
- `site_name` (string, required): Nama site
- `site_info` (string, required): Informasi site
- `pekerjaan` (string, required): Jenis pekerjaan
- `lokasi` (string, required): Lokasi site
- `nomor_kontrak` (string, required): Nomor kontrak
- `start` (string, required): Tanggal mulai (format: YYYY-MM-DD)
- `end` (string, required): Tanggal selesai (format: YYYY-MM-DD)
- `maximal_budget` (integer, required): Budget maksimal (Rupiah)
- `cost_estimated` (integer, required): Estimasi biaya (Rupiah)
- `pemberi_tugas` (string, required): Pemberi tugas/klien
- `penerima_tugas` (string, required): Penerima tugas/kontraktor
- `site_document` (string, optional): URL/path dokumen site
- `team_members` (array, optional): Array of people IDs untuk team

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "sites:73tnamhln5s1oehr2om2",
    "project_id": "projects:b7v5e43bvtpwyipxlemg",
    "site_name": "Site Menteng",
    "site_info": "Area Menteng Jakarta Pusat dengan 500 rumah",
    "pekerjaan": "Instalasi Fiber to Home",
    "lokasi": "Menteng, Jakarta Pusat",
    "nomor_kontrak": "KTR/2026/001",
    "start": "2026-03-15",
    "end": "2026-07-15",
    "maximal_budget": 500000000,
    "cost_estimated": 450000000,
    "pemberi_tugas": "PT Telkom Indonesia",
    "penerima_tugas": "PT SmartElco Solutions",
    "site_document": null,
    "created_at": "2026-02-20T09:39:14.482378100Z",
    "updated_at": "2026-02-20T09:39:14.482411560Z"
  },
  "message": "Site created successfully"
}
```

**Note:** 
- Jika `team_members` diberikan, sistem akan otomatis create:
  - 1 record di table `teams` dengan nama "Team {site_name}"
  - N records di table `team_peoples` untuk setiap member

### List All Sites
**GET** `/sites`

**Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "sites:73tnamhln5s1oehr2om2",
      "project_id": "projects:b7v5e43bvtpwyipxlemg",
      "site_name": "Site Menteng",
      "site_info": "Area Menteng...",
      "pekerjaan": "Instalasi Fiber to Home",
      "lokasi": "Menteng, Jakarta Pusat",
      "nomor_kontrak": "KTR/2026/001",
      "start": "2026-03-15",
      "end": "2026-07-15",
      "maximal_budget": 500000000,
      "cost_estimated": 450000000,
      "pemberi_tugas": "PT Telkom Indonesia",
      "penerima_tugas": "PT SmartElco Solutions",
      "site_document": null,
      "created_at": "2026-02-20T09:39:14Z",
      "updated_at": "2026-02-20T09:39:14Z"
    }
  ],
  "message": null
}
```

### Get Site by ID
**GET** `/sites/:id`

**Path Parameters:**
- `id`: ID site (format: `sites:xxx` — dengan prefix tabel)

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "sites:73tnamhln5s1oehr2om2",
    "project_id": "projects:b7v5e43bvtpwyipxlemg",
    "site_name": "Site Menteng",
    "site_info": "Area Menteng...",
    "pekerjaan": "Instalasi Fiber to Home",
    "lokasi": "Menteng, Jakarta Pusat",
    "latitude": "-6.197500",
    "longitude": "106.832000",
    "nomor_kontrak": "KTR/2026/001",
    "start": "2026-03-15",
    "end": "2026-07-15",
    "maximal_budget": 500000000,
    "cost_estimated": 450000000,
    "pemberi_tugas": "PT Telkom Indonesia",
    "penerima_tugas": "PT SmartElco Solutions",
    "stage": "implementasi",
    "stage_updated_at": "2026-03-10T07:00:00Z",
    "stage_notes": "Pekerjaan dimulai",
    "impl_cico_done": false,
    "impl_rfs_done": false,
    "impl_dokumen_done": false,
    "ineom_registered": false,
    "site_document": null,
    "created_at": "2026-02-20T09:39:14Z",
    "updated_at": "2026-03-10T07:00:00Z"
  },
  "message": null
}
```

**Response (Site Tidak Ditemukan):**
```json
{
  "success": false,
  "data": null,
  "message": "Site tidak ditemukan"
}
```

---

### Get Sites by Project
**GET** `/sites/project/:project_id`

**Path Parameters:**
- `project_id`: ID project (contoh: `b7v5e43bvtpwyipxlemg` - tanpa prefix "projects:")

**Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "sites:73tnamhln5s1oehr2om2",
      "project_id": "projects:b7v5e43bvtpwyipxlemg",
      "site_name": "Site Menteng",
      // ... site data lengkap
    }
  ],
  "message": null
}
```

---

## � Tim Struktur (Site Team Structure) API

> **Konsep:** Data Master Team (`/api/teams`) berisi daftar karyawan/member yang telah diupload via Excel.
> Tim Struktur adalah relasi antara Data Master Team ↔ Site tertentu.
> Frontend: tampilkan list `/api/teams`, user klik member → POST ke `/api/sites/:id/team-structure`.

### Get Tim Struktur (List Members)
**GET** `/sites/:site_id/team-structure`

**Path Parameters:**
- `site_id`: ID site (format: `sites:xxx` atau hanya `xxx`)

**Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "site_team_members:lgfltwpe8zkktymmh843",
      "site_id": "sites:zz5gdau1wutgrgpc8we0",
      "team_master_id": "teams:p84takz9nl6ihpwm05a2",
      "role": "leader",
      "vendor": "Vendor A",
      "nik": "14175063",
      "nama": "RIVO HIDAYAT",
      "no_hp": "081284238948",
      "jabatan": "HEAD COORDINATOR",
      "regional": "JAKARTA",
      "created_at": "2026-03-06T07:36:03.277490049Z",
      "updated_at": "2026-03-06T07:36:03.277491049Z"
    },
    {
      "id": "site_team_members:2zqjo7mgymyhfv8jprrm",
      "site_id": "sites:zz5gdau1wutgrgpc8we0",
      "team_master_id": "teams:ill8s861h9w9dl5fbc8n",
      "role": "member",
      "vendor": "Vendor D",
      "nik": "14175062",
      "nama": "YUDIE RAHMAN",
      "no_hp": "081299934817",
      "jabatan": "PROJECT MANAGER",
      "regional": "JAKARTA",
      "created_at": "2026-03-06T07:36:18.677497826Z",
      "updated_at": "2026-03-06T07:36:18.677499826Z"
    }
  ],
  "message": null
}
```

**Field Response (dari Data Master Team):**
| Field | Sumber | Keterangan |
|---|---|---|
| `id` | `site_team_members` | ID relasi |
| `site_id` | `site_team_members` | ID site |
| `team_master_id` | `site_team_members` | ID record di Data Master Team |
| `role` | `site_team_members` | Role di tim ini (e.g. "leader", "member") |
| `vendor` | `site_team_members` | Nama vendor |
| `nik` | `teams` (master) | NIK karyawan |
| `nama` | `teams` (master) | Nama karyawan |
| `no_hp` | `teams` (master) | Nomor HP |
| `jabatan` | `teams` (master) | Jabatan kerja |
| `regional` | `teams` (master) | Regional |

---

### Add Member ke Tim Struktur
**POST** `/sites/:site_id/team-structure`

> **Flow:** Ambil list dari `GET /api/teams`, user pilih member, kirim `team_master_id` ke endpoint ini.

**Path Parameters:**
- `site_id`: ID site (format: `sites:xxx` atau hanya `xxx`)

**Request Body:**
```json
{
  "team_master_id": "teams:p84takz9nl6ihpwm05a2",
  "role": "leader",
  "vendor": "Vendor A"
}
```

**Field Request:**
- `team_master_id` (string, required): ID dari Data Master Team (`GET /api/teams` → ambil field `id`)
- `role` (string, optional): Role dalam Tim Struktur. Contoh: `"leader"`, `"member"`, `"supervisor"`
- `vendor` (string, optional): Nama vendor

**Response sukses (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "site_team_members:lgfltwpe8zkktymmh843",
    "site_id": "sites:zz5gdau1wutgrgpc8we0",
    "team_master_id": "teams:p84takz9nl6ihpwm05a2",
    "role": "leader",
    "vendor": "Vendor A",
    "nik": "14175063",
    "nama": "RIVO HIDAYAT",
    "no_hp": "081284238948",
    "jabatan": "HEAD COORDINATOR",
    "regional": "JAKARTA",
    "created_at": "2026-03-06T07:36:03.277490049Z",
    "updated_at": "2026-03-06T07:36:03.277491049Z"
  },
  "message": "Team member added to site structure successfully"
}
```

**Response duplikat (200 OK, success=false):**
```json
{
  "success": false,
  "data": null,
  "message": "Member already added to this site's team structure"
}
```

---

### Remove Member dari Tim Struktur
**DELETE** `/sites/:site_id/team-structure/:member_id`

> Menghapus relasi member dari Tim Struktur site. **Tidak** menghapus record dari Data Master Team.

**Path Parameters:**
- `site_id`: ID site (format: `sites:xxx` atau hanya `xxx`)
- `member_id`: ID record Tim Struktur (field `id` dari response list, format: `site_team_members:xxx`)

**Response (200 OK):**
```json
{
  "success": true,
  "data": null,
  "message": "Team member removed from site structure"
}
```

---

## �👥 People API

### Create Person
**POST** `/people`

**Request Body (Minimal):**
```json
{
  "name": "Budi Santoso",
  "no_hp": "081234567890",
  "email": "budi@smartelco.com",
  "jabatan_kerja": "Teknisi Senior",
  "pekerjaan": "Instalasi Fiber"
}
```

**Request Body (Lengkap - All Fields):**
```json
{
  "name": "Budi Santoso",
  "tanggal_lahir": "1990-05-15",
  "tempat_lahir": "Jakarta",
  "agama": "Islam",
  "jenis_kelamin": "Laki-laki",
  "no_ktp": "3174012345678901",
  "no_hp": "081234567890",
  "email": "budi@smartelco.com",
  "jabatan_kerja": "Teknisi Senior",
  "regional": "Jakarta",
  "lokasi_kerja": "Jakarta Pusat",
  "pekerjaan": "Instalasi Fiber"
}
```

**Field Definitions:**
- `name` (string, required): Nama lengkap
- `tanggal_lahir` (string, optional): Tanggal lahir (YYYY-MM-DD)
- `tempat_lahir` (string, optional): Tempat lahir
- `agama` (string, optional): Agama
- `jenis_kelamin` (string, optional): Jenis kelamin
- `no_ktp` (string, optional): Nomor KTP
- `no_hp` (string, optional): Nomor HP
- `email` (string, optional): Email
- `jabatan_kerja` (string, optional): Jabatan
- `regional` (string, optional): Regional
- `lokasi_kerja` (string, optional): Lokasi kerja
- `pekerjaan` (string, optional): Jenis pekerjaan

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "people:1q9t3fd5jiu07j1jl2jj",
    "name": "Budi Santoso",
    "tanggal_lahir": null,
    "tempat_lahir": null,
    "agama": null,
    "jenis_kelamin": null,
    "no_ktp": null,
    "no_hp": "081234567890",
    "email": "budi@smartelco.com",
    "jabatan_kerja": "Teknisi Senior",
    "regional": null,
    "lokasi_kerja": null,
    "pekerjaan": "Instalasi Fiber",
    "nama_kontak_darurat": null,
    "nomor_kontak_darurat": null,
    "alamat_kontak_darurat": null,
    "status_pernikahan": null,
    "nama_ibu_kandung": null,
    "pendidikan_terakhir": null,
    "nama_kampus_sekolah": null,
    "jurusan_sekolah": null,
    "tahun_lulus": null,
    "foto_ktp": null,
    "foto_diri": null,
    "thumbnail_path": null,
    "created_at": "2026-02-20T09:30:00Z",
    "updated_at": "2026-02-20T09:30:00Z"
  },
  "message": "Person created successfully"
}
```

### List All People
**GET** `/people`

**Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "people:1q9t3fd5jiu07j1jl2jj",
      "name": "Budi Santoso",
      "no_hp": "081234567890",
      "email": "budi@smartelco.com",
      "jabatan_kerja": "Teknisi Senior",
      "pekerjaan": "Instalasi Fiber",
      // ... all fields
    }
  ],
  "message": null
}
```

---

## � Costs API

### Create Cost
**POST** `/costs`

**Request Body:**
```json
{
  "project_id": "projects:b7v5e43bvtpwyipxlemg",
  "site_id": "sites:73tnamhln5s1oehr2om2",
  "type_termin": "Termin 1",
  "tgl_pengajuan": "2026-02-20",
  "jumlah_pengajuan": 100000000,
  "status": "pending",
  "catatan_tolak": null
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "costs:abc123",
    "project_id": "projects:b7v5e43bvtpwyipxlemg",
    "site_id": "sites:73tnamhln5s1oehr2om2",
    "type_termin": "Termin 1",
    "tgl_pengajuan": "2026-02-20",
    "jumlah_pengajuan": 100000000,
    "status": "pending",
    "created_at": "2026-02-20T10:00:00Z"
  },
  "message": "Cost created successfully"
}
```

### List All Costs
**GET** `/costs`

### Get Costs by Project
**GET** `/costs/project/:project_id`

### Get Costs by Site
**GET** `/costs/site/:site_id`

### Approve Cost
**POST** `/costs/:cost_id/approve`

**Request Body:**
```json
{
  "acc_by": "user:admin123",
  "acc_name": "John Doe",
  "jumlah_acc": 95000000
}
```

---

## 📦 Materials API

### Create Material
**POST** `/materials`

**Request Body:**
```json
{
  "skp": "SKP-2026-001",
  "name": "Kabel Fiber Optik 100m",
  "unit": "Roll",
  "qty": 50,
  "project_id": "projects:b7v5e43bvtpwyipxlemg",
  "site_id": "sites:73tnamhln5s1oehr2om2",
  "tgl": "2026-02-20"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "materials:xyz789",
    "skp": "SKP-2026-001",
    "name": "Kabel Fiber Optik 100m",
    "unit": "Roll",
    "qty": 50,
    "project_id": "projects:b7v5e43bvtpwyipxlemg",
    "site_id": "sites:73tnamhln5s1oehr2om2",
    "tgl": "2026-02-20",
    "created_at": "2026-02-20T10:00:00Z"
  },
  "message": "Material created successfully"
}
```

### List All Materials
**GET** `/materials`

### Get Materials by Project
**GET** `/materials/project/:project_id`

### Get Materials by Site
**GET** `/materials/site/:site_id`

---

## 🌍 Areas & Regions API

### Create Area
**POST** `/areas`

**Request Body:**
```json
{
  "nama_area": "Jakarta"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "areas:jkt001",
    "nama_area": "Jakarta",
    "created_at": "2026-02-20T10:00:00Z"
  },
  "message": "Area created successfully"
}
```

### List All Areas
**GET** `/areas`

### Create Region
**POST** `/regions`

**Request Body:**
```json
{
  "area_id": "areas:jkt001",
  "kode_region": "JKT-PUSAT",
  "nama_region": "Jakarta Pusat"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "regions:reg001",
    "area_id": "areas:jkt001",
    "kode_region": "JKT-PUSAT",
    "nama_region": "Jakarta Pusat",
    "created_at": "2026-02-20T10:00:00Z"
  },
  "message": "Region created successfully"
}
```

### List All Regions
**GET** `/regions`

### Get Regions by Area
**GET** `/regions/area/:area_id`

---

## 📁 File Management API

### Upload Project File
**POST** `/project-files`

**Request Body:**
```json
{
  "project_id": "projects:b7v5e43bvtpwyipxlemg",
  "title": "Project Documentation",
  "filename": "doc_2026_02_20.pdf",
  "original_name": "Project Plan.pdf",
  "bucket": "smartelco-files",
  "key": "projects/xyz/doc.pdf",
  "mime_type": "application/pdf",
  "size": 2048576,
  "disk": "s3",
  "visibility": "private"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "project_files:file001",
    "project_id": "projects:b7v5e43bvtpwyipxlemg",
    "title": "Project Documentation",
    "filename": "doc_2026_02_20.pdf",
    "original_name": "Project Plan.pdf",
    "size": 2048576,
    "uploaded_at": "2026-02-20T10:00:00Z"
  },
  "message": "Project file uploaded successfully"
}
```

### List Project Files
**GET** `/projects/:project_id/files`

### Delete Project File
**DELETE** `/project-files/:file_id/delete`

### Upload Site File
**POST** `/site-files`

**Request Body:** (sama seperti project file, tapi gunakan `site_id`)

### List Site Files
**GET** `/sites/:site_id/files`

### Delete Site File
**DELETE** `/site-files/:file_id/delete`

---

## 💵 Termins API

### Create Termin
**POST** `/termins`

**Request Body (Save as Draft):**
```json
{
  "project_id": "projects:b7v5e43bvtpwyipxlemg",
  "site_id": "sites:73tnamhln5s1oehr2om2",
  "type_termin": "TERMIN_2",
  "tgl_terima": "2026-02-20",
  "jumlah": 50000000,
  "termin_ke": 2,
  "percentage": 50,
  "status": "draft",
  "keterangan": "Pengajuan termin ke-2"
}
```

**Request Body (Direct Submit for Review):**
```json
{
  "project_id": "projects:b7v5e43bvtpwyipxlemg",
  "site_id": "sites:73tnamhln5s1oehr2om2",
  "type_termin": "TERMIN_1",
  "tgl_terima": "2026-02-15",
  "jumlah": 25000000,
  "termin_ke": 1,
  "percentage": 25,
  "keterangan": "Pengajuan termin ke-1",
  "submitted_by": "Budi Santoso"
}
```

**Field Definitions:**
- `project_id` (string, required): ID project (format: "projects:xxx")
- `site_id` (string, required): ID site (format: "sites:xxx")
- `type_termin` (string, required): Tipe termin (e.g., "TERMIN_1", "TERMIN_2")
- `tgl_terima` (string, optional): Tanggal terima (format: YYYY-MM-DD)
- `jumlah` (integer, required): Jumlah pembayaran termin dalam Rupiah - **FLEXIBLE, tidak harus match dengan percentage**
- `termin_ke` (integer, **REQUIRED**): Urutan termin (1, 2, 3, atau 4) - **WAJIB diisi**
- `percentage` (integer, **REQUIRED**): Persentase dari maximal_budget - **WAJIB sesuai pola: Termin 1=30%, Termin 2=50%, Termin 3=10%, Termin 4=10%**
- `status` (string, optional): Status termin (default: "draft")
- `keterangan` (string, optional): Keterangan tambahan
- `submitted_by` (string, optional): Nama pengaju. Jika diisi, termin langsung berstatus "pending_review"

**🔒 VALIDASI KETAT (Business Rules):**

1. **Pola Percentage Terkunci (Struktur):**
   - Termin 1 HARUS 30%
   - Termin 2 HARUS 50%
   - Termin 3 HARUS 10%
   - Termin 4 HARUS 10%
   - Tidak bisa menggunakan percentage lain. Request akan ditolak.
   - **CATATAN:** Percentage adalah label/struktur termin, bukan constraint jumlah pembayaran

2. **Urutan Sequential:**
   - Termin 2 hanya bisa dibuat setelah Termin 1 berstatus "approved" atau "paid"
   - Termin 3 hanya bisa dibuat setelah Termin 2 berstatus "approved" atau "paid"
   - Termin 4 hanya bisa dibuat setelah Termin 3 berstatus "approved" atau "paid"
   - Tidak bisa skip termin atau buat termin secara paralel

3. **Maksimal Pembayaran 70% (Hard Limit):**
   - Total semua termin (yang sudah ada + yang baru diajukan) tidak boleh melebihi 70% dari `maximal_budget` site
   - Contoh: Site budget 100 juta → maksimal total pembayaran 70 juta
   - Sistem akan menghitung total kumulatif dan menolak jika melebihi batas
   - **PENTING:** Karena limit 70%, Anda bisa adjust `jumlah` setiap termin agar fit dalam limit ini

4. **Jumlah Flexible:**
   - `jumlah` TIDAK wajib sama dengan `percentage × site.maximal_budget`
   - Field `percentage` adalah label/struktur termin (30-50-10-10), bukan constraint jumlah
   - Anda bebas mengisi `jumlah` berapa saja, selama total ≤ 70% dari site budget
   - **Contoh:** Site 100 juta, Termin 2 (50%) bisa diisi 20 juta atau 30 juta (tidak harus 50 juta)

**✅ Contoh Skenario yang DITERIMA:**

```json
// Site budget: 100 juta, Max total: 70 juta
// Termin 1 (30%): 25 juta → ✅ OK (not exactly 30 juta, but within 70% limit)
// Termin 2 (50%): 30 juta → ✅ OK (not exactly 50 juta, but total 55 juta < 70 juta)
// Termin 3 (10%): 10 juta → ✅ OK (total 65 juta < 70 juta)
// Termin 4 (10%): 5 juta  → ✅ OK (total 70 juta = exactly 70%)
```

**⚠️ Contoh Skenario yang DITOLAK:**

```json
// ❌ DITOLAK: Percentage tidak sesuai pola
{
  "termin_ke": 1,
  "percentage": 40,  // Seharusnya 30%
  "jumlah": 40000000
}
// Error: "Termin 1 harus memiliki percentage 30%, bukan 40%"

// ❌ DITOLAK: Termin 2 dibuat sebelum Termin 1 approved
{
  "termin_ke": 2,
  "percentage": 50,
  "jumlah": 50000000
}
// Error: "Termin 1 harus disetujui direktur terlebih dahulu"

// ❌ DITOLAK: Total melebihi 70%
// Site budget: 100 juta (max: 70 juta)
// Already: Termin 1 (25 jt) + Termin 2 (40 jt) = 65 juta
// New: Termin 3 (10 jt) → Total akan 75 juta > 70 juta ❌
// Error: "Total pembayaran (75000000) melebihi batas maksimal 70% (70000000)"
```

**Response (200 OK - Draft):**
```json
{
  "success": true,
  "data": {
    "id": "termins:abc123",
    "project_id": "projects:b7v5e43bvtpwyipxlemg",
    "site_id": "sites:73tnamhln5s1oehr2om2",
    "type_termin": "TERMIN_2",
    "tgl_terima": "2026-02-20",
    "jumlah": 50000000,
    "termin_ke": 2,
    "percentage": 50,
    "status": "draft",
    "keterangan": "Pengajuan termin ke-2",
    "submitted_by": null,
    "submitted_at": null,
    "reviewed_by": null,
    "reviewed_at": null,
    "catatan_review": null,
    "approved_by": null,
    "approved_at": null,
    "catatan_approval": null,
    "paid_by": null,
    "paid_at": null,
    "jumlah_dibayar": null,
    "catatan_pembayaran": null,
    "bukti_pembayaran": null,
    "created_at": "2026-02-25T10:00:00Z",
    "updated_at": "2026-02-25T10:00:00Z"
  },
  "message": "Termin created successfully"
}
```

**Response (200 OK - Direct Submit):**
```json
{
  "success": true,
  "data": {
    "id": "termins:xyz789",
    "status": "pending_review",
    "submitted_by": "Budi Santoso",
    "submitted_at": "2026-02-25T10:05:00Z",
    ...
  },
  "message": "Termin created successfully"
}
```

**Response (400 Validation Failed - Wrong Percentage Pattern):**
```json
{
  "success": false,
  "data": null,
  "message": "Validation failed: Termin 1 harus memiliki percentage 30%, bukan 40%. Pola yang benar: Termin 1=30%, Termin 2=50%, Termin 3=10%, Termin 4=10%"
}
```

**Response (400 Validation Failed - Previous Termin Not Approved):**
```json
{
  "success": false,
  "data": null,
  "message": "Validation failed: Termin 1 harus disetujui direktur (status: approved) terlebih dahulu sebelum mengajukan Termin 2. Status Termin 1 saat ini: pending_review"
}
```

**Response (400 Validation Failed - Exceeds 70% Limit):**
```json
{
  "success": false,
  "data": null,
  "message": "Validation failed: Total pembayaran (Rp 90000000) melebihi batas maksimal 70% dari nilai site (Rp 70000000). Total saat ini: Rp 80000000, Termin baru: Rp 10000000, Sisa kuota: Rp -10000000"
}
```

**Response (400 Validation Failed - Wrong Amount):**
```json
{
  "success": false,
  "data": null,
  "message": "Validation failed: jumlah (10000000) does not match expected amount (30000000) based on 30% of site maximal_budget (100000000)"
}
```

**Response (400 Validation Failed - Invalid termin_ke):**
```json
{
  "success": false,
  "data": null,
  "message": "Validation failed: termin_ke must be between 1-4. Got: 5"
}
```

### Get Termin by ID
**GET** `/termins/:termin_id`

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "termins:abc123",
    "project_id": "projects:xxx",
    "site_id": "sites:xxx",
    "type_termin": "TERMIN_1",
    "tgl_terima": "2026-02-15",
    "jumlah": 25000000,
    "termin_ke": 1,
    "percentage": 25,
    "status": "approved",
    ...
  },
  "message": null
}
```

### List All Termins
**GET** `/termins`

**Response:** Array of termins, sorted by `created_at DESC`

**✅ v1.7.1 Enhancement:** Response includes enriched `site_id` field with both `site_name` and `project_name`

**Response Example:**
```json
{
  "success": true,
  "data": [
    {
      "id": "termins:xxx",
      "project_id": "projects:yyy",
      "site_id": {
        "site_name": "Site Jakarta Selatan",
        "project_name": "Network Expansion Jakarta"
      },
      "termin_ke": 1,
      "percentage": 30,
      "status": "draft",
      "jumlah": 15000000,
      ...
    }
  ]
}
```

### Get Termins by Project
**GET** `/termins/project/:project_id`

**Response:** Array of termins for specific project

**✅ v1.7.1 Enhancement:** Response includes enriched `site_id` field with both `site_name` and `project_name`

### Get Termins by Site
**GET** `/termins/site/:site_id`

**Response:** Array of termins for specific site

**✅ v1.7.1 Enhancement:** Response includes enriched `site_id` field with both `site_name` and `project_name`

### Update Termin (Draft Only)
**PUT** `/termins/:termin_id`

**Note:** Hanya termin dengan status "draft" yang bisa diupdate.

**Request Body:**
```json
{
  "type_termin": "TERMIN_1_UPDATED",
  "tgl_terima": "2026-02-16",
  "jumlah": 26000000,
  "keterangan": "Updated keterangan"
}
```

### Delete Termin
**DELETE** `/termins/:termin_id`

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Termin and associated files deleted successfully"
}
```

### Submit Termin for Review
**POST** `/termins/:termin_id/submit`

**Request Body:**
```json
{
  "submitter_name": "Ahmad Santoso"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "termins:abc123",
    "status": "pending_review",
    "submitted_by": "Ahmad Santoso",
    "submitted_at": "2026-02-25T11:00:00Z",
    ...
  },
  "message": "Termin submitted for review"
}
```

### Review Termin (Field Head)
**POST** `/termins/:termin_id/review`

**Request Body:**
```json
{
  "reviewer_name": "Indra Sadik",
  "catatan_review": "Progress pekerjaan sudah sesuai. Disetujui untuk diteruskan ke direktur.",
  "approve": true
}
```

**Field Definitions:**
- `reviewer_name` (string, required): Nama Field Head yang mereview
- `catatan_review` (string, required): Catatan hasil review
- `approve` (boolean, required): 
  - `true` = Approve, status menjadi "reviewed"
  - `false` = Reject, status kembali ke "draft"

**Response (200 OK - Approved):**
```json
{
  "success": true,
  "data": {
    "id": "termins:abc123",
    "status": "reviewed",
    "reviewed_by": "Indra Sadik",
    "reviewed_at": "2026-02-25T12:00:00Z",
    "catatan_review": "Progress pekerjaan sudah sesuai...",
    ...
  },
  "message": "Termin reviewed and approved by Field Head. Waiting for Director approval."
}
```

**Response (200 OK - Rejected):**
```json
{
  "success": true,
  "data": {
    "status": "draft",
    "reviewed_by": "Indra Sadik",
    "catatan_review": "Pekerjaan belum sesuai, silahkan perbaiki",
    ...
  },
  "message": "Termin rejected by Field Head. Returned to draft."
}
```

### Approve Termin (Director)
**POST** `/termins/:termin_id/approve`

**Request Body:**
```json
{
  "approver_name": "Direktur Utama",
  "catatan_approval": "Termin disetujui oleh direktur. Silahkan proses pembayaran.",
  "approve": true
}
```

**Field Definitions:**
- `approver_name` (string, required): Nama Direktur yang menyetujui
- `catatan_approval` (string, required): Catatan persetujuan
- `approve` (boolean, required):
  - `true` = Approve, status menjadi "approved"
  - `false` = Reject, status kembali ke "draft"

**Response (200 OK - Approved):**
```json
{
  "success": true,
  "data": {
    "status": "approved",
    "approved_by": "Direktur Utama",
    "approved_at": "2026-02-25T13:00:00Z",
    "catatan_approval": "Termin disetujui...",
    ...
  },
  "message": "Termin approved by Director. Waiting for payment by Finance."
}
```

### Pay Termin (Finance)
**POST** `/termins/:termin_id/pay`

**Content-Type:** `multipart/form-data`

⚠️ **PERUBAHAN PENTING:** Endpoint ini sekarang menggunakan **multipart/form-data** untuk upload file bukti pembayaran. File akan **disimpan langsung ke database SurrealDB sebagai base64**, bukan ke disk!

**Request Body (Form Data):**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `approved_by` | text | ✅ Yes | ID user yang login yang meng-approve pembayaran (tanpa prefix "users:"). Contoh: "7lwx51qk56xe13arlctl" |
| `jumlah_dibayar` | text | ✅ Yes | Jumlah yang dibayarkan dalam Rupiah (angka, e.g., "25000000") |
| `referensi_pembayaran` | text | ✅ Yes | Nomor referensi pembayaran seperti nomor transfer, cek (e.g., "TRF-12345B", "CEK-001") |
| `catatan_pembayaran` | text | ❌ No | Catatan/keterangan tambahan pembayaran |
| `bukti_pembayaran` | file | ❌ No | **File upload** bukti pembayaran (PDF, JPG, PNG, dll). File akan di-encode ke base64 dan disimpan di database |

**Field Definitions:**
- `approved_by` (text, required): **ID user yang login** yang meng-approve pembayaran (tanpa prefix "users:")
  - Sistem akan otomatis ambil nama dan email dari user ID ini
  - Tidak ada validasi role khusus, user dengan role apapun bisa approve pembayaran
  - Contoh: "7lwx51qk56xe13arlctl"
- `jumlah_dibayar` (text, required): Jumlah yang dibayarkan (dalam Rupiah, input sebagai text/string)
- `referensi_pembayaran` (text, required): **Nomor referensi pembayaran** seperti nomor transfer, nomor cek, dll (e.g., TRF-12345B, INV-001, CEK-2026-001)
- `catatan_pembayaran` (text, optional): Catatan/keterangan pembayaran
- `bukti_pembayaran` (file, optional): **Upload file bukti pembayaran** langsung (screenshot transfer, PDF, image, dll)
  - **File akan di-encode ke base64 dan disimpan langsung ke database**
  - Response akan berisi base64 string yang bisa di-decode kembali ke file original
  - Metadata file (filename, MIME type, size) juga disimpan di database
  - Supported formats: PDF, JPG, JPEG, PNG, TXT, dll

**Example cURL:**
```bash
curl -X POST http://localhost:3000/api/termins/ak0opm1rih5ttaoowc29/pay \
  -F "approved_by=7lwx51qk56xe13arlctl" \
  -F "jumlah_dibayar=25000000" \
  -F "referensi_pembayaran=TRF-12345B" \
  -F "catatan_pembayaran=Pembayaran termin 1 via transfer BCA" \
  -F "bukti_pembayaran=@/path/to/payment-proof.pdf"
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "status": "paid",
    "paid_by": "Finance Manager (finance@smartelco.com)",
    "paid_at": "2026-02-27T14:00:00Z",
    "jumlah_dibayar": 25000000,
    "referensi_pembayaran": "TRF-12345B",
    "catatan_pembayaran": "Pembayaran termin 1...",
    "bukti_pembayaran": "JVBERi0xLjQKMSAwIG9iajw8L1R5cGUvQ2F0YWxvZy...",
    "bukti_pembayaran_filename": "payment-proof.pdf",
    "bukti_pembayaran_mime_type": "application/pdf",
    "bukti_pembayaran_size": 123456,
    ...
  },
  "message": "Payment confirmed. Termin completed."
}
```

**Field Explanations (Response):**
- `bukti_pembayaran`: Base64 encoded file content (untuk download/view, decode base64 ini kembali ke file)
- `bukti_pembayaran_filename`: Nama file original yang di-upload
- `bukti_pembayaran_mime_type`: MIME type file (e.g., "application/pdf", "image/jpeg")
- `bukti_pembayaran_size`: Ukuran file dalam bytes

**Response (404 Not Found - User Not Found):**
```json
{
  "success": false,
  "data": null,
  "message": "User not found"
}
```

**Response (400 Bad Request - Missing Required Fields):**
```json
{
  "success": false,
  "data": null,
  "message": "Bad Request"
}
```

### 🔄 Termin Workflow Summary

```
DRAFT 
  ↓ (submit)
PENDING_REVIEW
  ↓ (Field Head review: approve=true)
REVIEWED
  ↓ (Director approve: approve=true)
APPROVED
  ↓ (Finance pay)
PAID

Note: Jika di-reject di tahap manapun (approve=false), status kembali ke DRAFT.
```

### Upload Termin File
**POST** `/termin-files`

**Request Body:**
```json
{
  "termin_id": "termins:abc123",
  "category": "invoice",
  "title": "Invoice Termin 1",
  "filename": "invoice_term1.pdf",
  "original_name": "Invoice.pdf",
  "bucket": "smartelco-files",
  "key": "termins/xyz/invoice.pdf",
  "mime_type": "application/pdf",
  "size": 1048576,
  "disk": "s3",
  "visibility": "private"
}
```

### List Termin Files
**GET** `/termins/:termin_id/files`

### Delete Termin File
**DELETE** `/termin-files/:file_id/delete`

---

## �🔧 Health Check

### Server Health
**GET** `/health`

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Server is running",
  "timestamp": "2026-02-20T09:00:00.123456+00:00"
}
```

---

## 📊 Database Structure (SurrealDB)

### Tables
- **projects**: Project data dengan value (budget) dan cost tracking
- **sites**: Site/lokasi pekerjaan dalam project
- **people**: Data personel/karyawan lengkap
- **teams**: Team yang assigned ke project/site
- **team_peoples**: Junction table (many-to-many) antara teams dan people
- **costs**: Tracking pengeluaran/biaya per project/site dengan approval flow
- **materials**: Material yang digunakan dalam project/site
- **areas**: Area geografis (Jakarta, Surabaya, dll)
- **regions**: Regional dalam area (Jakarta Pusat, Jakarta Selatan, dll)
- **project_files**: File/dokumen yang terkait dengan project
- **site_files**: File/dokumen yang terkait dengan site
- **termins**: Termin pembayaran per project/site
- **termin_files**: File/dokumen termin (invoice, bukti pembayaran, dll)
- **users**: User management untuk aplikasi

### Relationships
```
projects (1) ---< sites (many)
projects (1) ---< teams (many)
projects (1) ---< costs (many)
projects (1) ---< materials (many)
projects (1) ---< termins (many)
projects (1) ---< project_files (many)

sites (1) ---< teams (many)
sites (1) ---< costs (many)
sites (1) ---< materials (many)
sites (1) ---< termins (many)
sites (1) ---< site_files (many)

teams (1) >--- leader (1 people)
teams (many) ---< team_peoples >--- (many) people

areas (1) ---< regions (many)

termins (1) ---< termin_files (many)
```

### ID Format
Semua ID menggunakan format SurrealDB: `table:random_id`
- Project ID: `projects:b7v5e43bvtpwyipxlemg`
- Site ID: `sites:73tnamhln5s1oehr2om2`
- People ID: `people:1q9t3fd5jiu07j1jl2jj`
- Team ID: `teams:z0wh2zubjkw3wxsh9mr7`

---

## 🚨 Error Responses

### 400 Bad Request
```json
{
  "success": false,
  "data": null,
  "message": "Invalid request data"
}
```

### 500 Internal Server Error
```json
{
  "success": false,
  "data": null,
  "message": "Database error: ..."
}
```

---

## 💡 Tips untuk Frontend Integration

### 1. URL Construction
```typescript
const BASE_URL = 'http://localhost:3000/api';

// Create project
fetch(`${BASE_URL}/projects`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(projectData)
})
```

### 2. ID Parsing
```typescript
// Extract ID dari response
const project = await response.json();
const projectId = project.data.id; // "projects:xxx"

// Use untuk create site
const siteData = {
  project_id: projectId,  // Langsung pakai format "projects:xxx"
  // ... field lainnya
}
```

### 3. Team Members Array
```typescript
// Collect selected people IDs
const selectedPeopleIds = [
  "people:1q9t3fd5jiu07j1jl2jj",
  "people:3h6pq9fhkkmshx7tyksz"
];

// Include in site creation
const siteData = {
  // ... site fields
  team_members: selectedPeopleIds  // Optional
}
```

### 4. Number Formatting
```typescript
// Budget harus dikirim sebagai integer (Rupiah)
const budget = 500_000_000; // 500 juta

// Jangan gunakan string atau float
❌ "500000000"
❌ 500000000.00
✅ 500000000
```

### 5. Date Format
```typescript
// Date harus dalam format YYYY-MM-DD
const startDate = "2026-03-15";  // ✅ Correct
const startDate = "15/03/2026";  // ❌ Wrong
```

---

## 📱 Example: Complete Flow

### Frontend Flow untuk Create Site dengan Team

```typescript
// 1. Get list of projects (untuk dropdown)
const projectsRes = await fetch(`${BASE_URL}/projects`);
const projects = await projectsRes.json();

// 2. Get list of people (untuk team member selection)
const peopleRes = await fetch(`${BASE_URL}/people`);
const people = await peopleRes.json();

// 3. User fills form and selects team members
const formData = {
  project_id: "projects:b7v5e43bvtpwyipxlemg", // from dropdown
  site_name: "Site Menteng",
  site_info: "Area dengan 500 rumah",
  pekerjaan: "Instalasi Fiber to Home",
  lokasi: "Menteng, Jakarta",
  nomor_kontrak: "KTR/2026/001",
  start: "2026-03-15",
  end: "2026-07-15",
  maximal_budget: 500000000,
  cost_estimated: 450000000,
  pemberi_tugas: "PT Telkom Indonesia",
  penerima_tugas: "PT SmartElco Solutions",
  site_document: null,
  team_members: [
    "people:1q9t3fd5jiu07j1jl2jj",
    "people:3h6pq9fhkkmshx7tyksz"
  ]
};

// 4. Create site
const siteRes = await fetch(`${BASE_URL}/sites`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(formData)
});

const result = await siteRes.json();
if (result.success) {
  console.log("Site created:", result.data.id);
  // Team juga otomatis ter-create!
}
```

---

## 🔄 What Happens Behind the Scenes

Ketika create site dengan `team_members`:

1. ✅ Create 1 record di table `sites`
2. ✅ Create 1 record di table `teams` dengan nama "Team {site_name}"
3. ✅ Create N records di table `team_peoples` (1 untuk setiap member)

Data final di database:
```
sites:
  id: sites:xxx
  project_id: projects:yyy
  site_name: "Site Menteng"
  ...

teams:
  id: teams:zzz
  nama: "Team Site Menteng"
  project_id: projects:yyy
  site_id: sites:xxx
  active: true

team_peoples:
  - team_id: teams:zzz, people_id: people:aaa
  - team_id: teams:zzz, people_id: people:bbb
```

---

---

## 📍 Site Stage Management API

> **Konsep:** Setiap site memiliki `stage` yang merepresentasikan progress pekerjaan.
> Update stage akan otomatis mencatat audit log di tabel `site_stage_log`.

### Update Site Stage
**POST** `/sites/:id/stage`

**Path Parameters:**
- `id`: ID site (format: `sites:xxx` atau hanya `xxx`)

**Valid Stage Values (berurutan):**
```
imported → assigned → permit_process → permit_ready →
akses_process → akses_ready → implementasi →
rfi_done → rfs_done → dokumen_done → bast → invoice → completed
```

**Deskripsi Tiap Stage:**
| # | Stage | Deskripsi |
|---|---|---|
| 01 | `imported` | Data site baru diimport, belum diproses |
| 02 | `assigned` | Tim lapangan sudah ditugaskan ke site |
| 03 | `permit_process` | Proses perizinan ke warga/RT/RW sedang berjalan |
| 04 | `permit_ready` | Dokumen izin sudah selesai dan ditandatangani |
| 05 | `akses_process` | SKP dibuat, menunggu material dikeluarkan gudang |
| 06 | `akses_ready` | Material sudah diterima tim lapangan di lokasi |
| 07 | `implementasi` | Pekerjaan fisik lapangan sedang berlangsung |
| 08 | `rfi_done` | Request For Inspection sudah dilakukan |
| 09 | `rfs_done` | Ready For Service dikonfirmasi, layanan siap aktif |
| 10 | `dokumen_done` | As-built drawing & seluruh dokumen sudah lengkap |
| 11 | `bast` | Berita Acara Serah Terima sudah ditandatangani |
| 12 | `invoice` | Invoice sudah diajukan ke finance client |
| 13 | `completed` | Seluruh pekerjaan dan administrasi selesai |

**Request Body:**
```json
{
  "stage": "implementasi",
  "notes": "Implementasi dimulai hari ini",
  "changed_by": "Budi Santoso",
  "evidence_urls": ["https://storage.example.com/foto1.jpg"],
  "impl_cico_done": false,
  "impl_rfs_done": false,
  "impl_dokumen_done": false,
  "ineom_registered": false
}
```

**Field Definitions:**
- `stage` (string, required): Target stage baru (harus salah satu dari 13 stage valid)
- `notes` (string, optional): Catatan perubahan stage
- `changed_by` (string, optional): Nama/ID user yang mengubah (default: "system")
- `evidence_urls` (array string, optional): URL foto/dokumen pendukung
- `impl_cico_done` (boolean, optional): Cico sudah selesai (relevan saat stage implementasi)
- `impl_rfs_done` (boolean, optional): RFS sudah selesai
- `impl_dokumen_done` (boolean, optional): Dokumen implementasi sudah selesai
- `ineom_registered` (boolean, optional): iNeOM sudah didaftarkan

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "sites:73tnamhln5s1oehr2om2",
    "project_id": "projects:b7v5e43bvtpwyipxlemg",
    "site_name": "Site Menteng",
    "stage": "implementasi",
    "stage_updated_at": "2026-03-10T07:00:00Z",
    "stage_notes": "Implementasi dimulai hari ini",
    "impl_cico_done": false,
    "impl_rfs_done": false,
    "impl_dokumen_done": false,
    "ineom_registered": false
  },
  "message": "Stage berhasil diupdate ke 'implementasi'"
}
```

**Response (Error - Invalid Stage):**
```json
{
  "success": false,
  "data": null,
  "message": "Stage 'invalid_stage' tidak valid"
}
```

---

### Test Scenario — Full Stage Flow (13 Stages)

Jalankan request berikut secara berurutan untuk menguji full lifecycle site dari awal hingga selesai:

```bash
BASE="http://localhost:3001/api"
SITE_ID="sites:xxx"

for STAGE in imported assigned permit_process permit_ready akses_process akses_ready implementasi rfi_done rfs_done dokumen_done bast invoice completed; do
  curl -s -X POST "$BASE/sites/$SITE_ID/stage" \
    -H "Content-Type: application/json" \
    -d "{\"stage\":\"$STAGE\",\"notes\":\"Test stage $STAGE\",\"changed_by\":\"Tester\"}" \
    | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'  {d[\"data\"][\"stage\"]} → OK' if d.get('success') else f'  FAIL: {d.get(\"message\")}')"
done
```

**Hasil yang diharapkan:** Semua 13 stage ter-update dengan `success: true`, masing-masing menghasilkan satu entry di `site_stage_log`.

---

### Get Site Stage Logs (Audit Trail)
**GET** `/sites/:id/stage-logs`

**Path Parameters:**
- `id`: ID site (format: `sites:xxx` atau hanya `xxx`)

**Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "site_stage_log:abc123",
      "site_id": "sites:73tnamhln5s1oehr2om2",
      "from_stage": "assigned",
      "to_stage": "implementasi",
      "notes": "Implementasi dimulai hari ini",
      "changed_by": "Budi Santoso",
      "evidence_urls": ["https://storage.example.com/foto1.jpg"],
      "created_at": "2026-03-10T07:00:00Z"
    }
  ],
  "message": null
}
```

---

## 📦 Site BOQ (Bill of Quantity) API

> **Konsep:** Daftar material/jasa yang tercantum dalam kontrak untuk satu site.
> Digunakan pada tab "Material Item" di halaman Detail Site.

### List BOQ by Site
**GET** `/sites/:site_id/boq`

**Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "site_boq:abc123",
      "site_id": "sites:73tnamhln5s1oehr2om2",
      "item_code": "MAT-001",
      "description": "Kabel Fiber Optik SM 12C",
      "quantity": 100.0,
      "unit": "meter",
      "type": "material",
      "source": "Warehouse Jakarta",
      "created_at": "2026-03-10T07:00:00Z",
      "updated_at": "2026-03-10T07:00:00Z"
    }
  ],
  "message": null
}
```

---

### Create BOQ Item
**POST** `/sites/:site_id/boq`

**Request Body:**
```json
{
  "item_code": "MAT-001",
  "description": "Kabel Fiber Optik SM 12C",
  "quantity": 100.0,
  "unit": "meter",
  "type": "material",
  "source": "Warehouse Jakarta"
}
```

**Field Definitions:**
- `item_code` (string, required): Kode item
- `description` (string, required): Deskripsi item
- `quantity` (float, required): Jumlah
- `unit` (string, required): Satuan (meter, unit, rol, dll)
- `type` (string, optional): `"material"` atau `"jasa"` (default: `"material"`)
- `source` (string, optional): Sumber/lokasi material

**Response (200 OK):**
```json
{
  "success": true,
  "data": { /* SiteBoq object */ },
  "message": "BOQ item created successfully"
}
```

---

### Update BOQ Item
**PUT** `/site-boq/:boq_id`

**Request Body (semua field optional):**
```json
{
  "description": "Kabel Fiber Optik SM 24C",
  "quantity": 150.0
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": { /* updated SiteBoq object */ },
  "message": "BOQ item updated successfully"
}
```

---

### Delete BOQ Item
**DELETE** `/site-boq/:boq_id`

**Response (200 OK):**
```json
{
  "success": true,
  "data": null,
  "message": "BOQ item deleted successfully"
}
```

---

## 📋 SKP (Surat Perintah Ambil Material) API

> **Konsep:** SKP adalah dokumen resmi izin pengambilan material dari gudang.
> Flow status: `Draft → Submitted → Received`

### List SKP by Site
**GET** `/sites/:site_id/skp`

**Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "skp:abc123",
      "site_id": "sites:73tnamhln5s1oehr2om2",
      "skp_number": "SKP/2026/001",
      "tanggal": "2026-03-10",
      "keterangan": "Pengambilan material batch 1",
      "status": "Draft",
      "uploaded_by": "Budi Santoso",
      "document_url": null,
      "received_proof_url": null,
      "created_at": "2026-03-10T07:00:00Z",
      "updated_at": "2026-03-10T07:00:00Z"
    }
  ],
  "message": null
}
```

---

### Create SKP
**POST** `/sites/:site_id/skp`

**Request Body:**
```json
{
  "skp_number": "SKP/2026/001",
  "tanggal": "2026-03-10",
  "keterangan": "Pengambilan material batch 1",
  "uploaded_by": "Budi Santoso",
  "document_url": null
}
```

**Field Definitions:**
- `skp_number` (string, required): Nomor SKP (unik)
- `tanggal` (string, required): Tanggal SKP (YYYY-MM-DD)
- `keterangan` (string, optional): Keterangan/deskripsi
- `uploaded_by` (string, required): Nama/ID yang mengupload
- `document_url` (string, optional): URL dokumen SKP

**Note:** Status awal selalu `"Draft"` secara otomatis.

**Response (200 OK):**
```json
{
  "success": true,
  "data": { /* Skp object */ },
  "message": "SKP created successfully"
}
```

---

### Get SKP by ID
**GET** `/skp/:skp_id`

**Response (200 OK):**
```json
{
  "success": true,
  "data": { /* Skp object */ },
  "message": null
}
```

---

### Update SKP
**PUT** `/skp/:skp_id`

**Request Body (semua field optional):**
```json
{
  "status": "Submitted",
  "document_url": "https://storage.example.com/skp-001.pdf"
}
```

**Valid status values:** `"Draft"`, `"Submitted"`, `"Received"`

**Field untuk update received proof:**
```json
{
  "status": "Received",
  "received_proof_url": "https://storage.example.com/bukti-terima-001.jpg"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": { /* updated Skp object */ },
  "message": "SKP updated successfully"
}
```

---

### Delete SKP
**DELETE** `/skp/:skp_id`

**Response (200 OK):**
```json
{
  "success": true,
  "data": null,
  "message": "SKP deleted successfully"
}
```

---

## 📸 Site Evidence (Foto Lapangan) API

> **Konsep:** Foto-foto dokumentasi lapangan yang diupload per tag progress.
> Tag digunakan untuk mengelompokkan foto berdasarkan tahap pekerjaan.

### List Evidence by Site
**GET** `/sites/:site_id/evidence`

**Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "site_evidence:abc123",
      "site_id": "sites:73tnamhln5s1oehr2om2",
      "filename": "foto_implementasi_001.jpg",
      "original_name": "IMG_20260310_070000.jpg",
      "file_url": "https://storage.example.com/foto_implementasi_001.jpg",
      "mime_type": "image/jpeg",
      "file_size": 1048576,
      "progress_tag": "implementasi",
      "stage_context": "Pemasangan tiang ODC area Menteng",
      "uploaded_by": "Budi Santoso",
      "uploaded_at": "2026-03-10T07:00:00Z"
    }
  ],
  "message": null
}
```

---

### Create Evidence (Upload Metadata)
**POST** `/sites/:site_id/evidence`

**Request Body:**
```json
{
  "filename": "foto_implementasi_001.jpg",
  "original_name": "IMG_20260310_070000.jpg",
  "file_url": "https://storage.example.com/foto_implementasi_001.jpg",
  "mime_type": "image/jpeg",
  "file_size": 1048576,
  "progress_tag": "implementasi",
  "stage_context": "Pemasangan tiang ODC area Menteng",
  "uploaded_by": "Budi Santoso"
}
```

**Field Definitions:**
- `filename` (string, required): Nama file tersimpan
- `original_name` (string, optional): Nama file asli dari device
- `file_url` (string, optional): URL file di storage
- `mime_type` (string, optional): Tipe MIME (image/jpeg, image/png, dll)
- `file_size` (integer, optional): Ukuran file dalam bytes
- `progress_tag` (string, required): Tag progress (contoh: `"implementasi"`, `"permit_process"`)
- `stage_context` (string, optional): Deskripsi konteks/keterangan foto
- `uploaded_by` (string, required): Nama/ID yang mengupload

**Response (200 OK):**
```json
{
  "success": true,
  "data": { /* SiteEvidence object */ },
  "message": "Evidence uploaded successfully"
}
```

---

### Delete Evidence
**DELETE** `/site-evidence/:evidence_id`

**Response (200 OK):**
```json
{
  "success": true,
  "data": null,
  "message": "Evidence deleted successfully"
}
```

---

## � Site Issue / Blocker API

> **Konsep:** Ketika ada masalah/blocker yang menghambat progress site, tim dapat melaporkannya.
> Ada dua jenis tindakan:
> - **Tahan di stage ini** (`tahan`): issue dicatat, site tetap di stage saat ini, status `open`.
> - **Eskalasi ke management** (`eskalasi`): issue dieskalasi, status langsung `escalated`.
> Issue dapat di-resolve setelah masalah diselesaikan.

### List Issues by Site
**GET** `/sites/:site_id/issues`

**Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "site_issue:abc123",
      "site_id": "sites:73tnam...",
      "stage_at_report": "implementasi",
      "keterangan": "Material tidak sesuai spesifikasi kontrak",
      "tindakan": "tahan",
      "status": "open",
      "reported_by": "Budi Santoso",
      "evidence_urls": ["https://cdn.example.com/bukti.jpg"],
      "resolved_by": null,
      "resolved_notes": null,
      "resolved_at": null,
      "created_at": "2026-03-11T05:31:00Z",
      "updated_at": "2026-03-11T05:31:00Z"
    }
  ],
  "message": null
}
```

---

### Create / Laporkan Issue
**POST** `/sites/:site_id/issues`

**Request Body:**
```json
{
  "stage_at_report": "implementasi",
  "keterangan": "Material tidak sesuai spesifikasi kontrak",
  "tindakan": "tahan",
  "reported_by": "Budi Santoso",
  "evidence_urls": ["https://cdn.example.com/bukti.jpg"]
}
```

**Field Definitions:**
- `stage_at_report` (string, required): Stage saat issue dilaporkan
- `keterangan` (string, required): Deskripsi masalah secara detail
- `tindakan` (string, required): `"tahan"` atau `"eskalasi"`
- `reported_by` (string, optional): Nama pelapor
- `evidence_urls` (array string, optional): URL foto/dokumen bukti issue

**Response (200 OK) — tindakan: tahan:**
```json
{
  "success": true,
  "data": {
    "id": "site_issue:abc123",
    "stage_at_report": "implementasi",
    "tindakan": "tahan",
    "status": "open",
    ...
  },
  "message": "Issue dilaporkan. Tindakan: tahan"
}
```

**Response (200 OK) — tindakan: eskalasi:**
```json
{
  "success": true,
  "data": {
    "tindakan": "eskalasi",
    "status": "escalated",
    ...
  },
  "message": "Issue dilaporkan. Tindakan: eskalasi"
}
```

**Response (Error — tindakan tidak valid):**
```json
{
  "success": false,
  "data": null,
  "message": "Tindakan harus 'tahan' atau 'eskalasi'"
}
```

---

### Get Issue by ID
**GET** `/site-issues/:issue_id`

**Response:** SiteIssue object lengkap (sama seperti item dalam list).

---

### Resolve Issue
**POST** `/site-issues/:issue_id/resolve`

**Request Body:**
```json
{
  "resolved_by": "Supervisor Tim",
  "resolved_notes": "Material sudah diganti sesuai spesifikasi"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "status": "resolved",
    "resolved_by": "Supervisor Tim",
    "resolved_notes": "Material sudah diganti sesuai spesifikasi",
    "resolved_at": "2026-03-11T07:00:00Z",
    ...
  },
  "message": "Issue berhasil di-resolve"
}
```

---

### Delete Issue
**DELETE** `/site-issues/:issue_id`

**Response (200 OK):**
```json
{
  "success": true,
  "data": null,
  "message": "Issue berhasil dihapus"
}
```

---

## �📊 Site Stage Reference

| Stage | UI Step | Deskripsi |
|---|---|---|
| `imported` | Step 1 | Site baru diimport/didaftarkan |
| `assigned` | Step 1 | Tim sudah diassign ke site |
| `permit_process` | Step 2 | Proses perizinan sedang berjalan |
| `permit_ready` | Step 2 | Perizinan selesai |
| `akses_process` | Step 3 | Proses akses lokasi |
| `akses_ready` | Step 3 | Akses lokasi sudah siap |
| `implementasi` | Step 4 | Pekerjaan implementasi berlangsung |
| `rfi_done` | Step 4 | RFI (Request for Inspection) selesai |
| `rfs_done` | Step 5 | RFS (Ready for Service) selesai |
| `dokumen_done` | Step 5 | Dokumen selesai |
| `bast` | Step 6 | BAST (Berita Acara Serah Terima) |
| `invoice` | Step 7 | Invoice diterbitkan |
| `completed` | Step 7 | Pekerjaan selesai |

---

**🚀 Server:** `http://localhost:3001`  
**📅 Last Updated:** March 11, 2026
