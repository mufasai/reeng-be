# 📚 Reengineering Tool Backend - API Documentation

**Base URL:** `http://localhost:3000/api`

---

## 📋 Changelog

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

## �🔐 Authentication

### Login
**POST** `/auth/login`

**Request Body:**
```json
{
  "email": "admin@smartelco.com",
  "password": "admin123"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "email": "admin@smartelco.com",
    "nama": "Administrator",
    "role": "admin"
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

## 👥 People API

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

### Get Termins by Project
**GET** `/termins/project/:project_id`

**Response:** Array of termins for specific project

### Get Termins by Site
**GET** `/termins/site/:site_id`

**Response:** Array of termins for specific site

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

**Request Body:**
```json
{
  "payer_name": "Siti Nurhaliza (Finance)",
  "jumlah_dibayar": 25000000,
  "referensi_pembayaran": "TRF-12345B",
  "catatan_pembayaran": "Pembayaran termin 1 via transfer BCA. Invoice #INV-2026-0425",
  "bukti_pembayaran": "https://storage.smartelco.com/bukti/termin1-payment.pdf"
}
```

**Field Definitions:**
- `payer_name` (string, required): Nama petugas Finance yang memproses
- `jumlah_dibayar` (integer, required): Jumlah yang dibayarkan (dalam Rupiah)
- `referensi_pembayaran` (string, required): Nomor referensi pembayaran (e.g., TRF-12345B, INV-001, dst)
- `catatan_pembayaran` (string, optional): Catatan/keterangan pembayaran
- `bukti_pembayaran` (string, optional): URL/path bukti pembayaran

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "status": "paid",
    "paid_by": "Siti Nurhaliza (Finance)",
    "paid_at": "2026-02-25T14:00:00Z",
    "jumlah_dibayar": 25000000,
    "referensi_pembayaran": "TRF-12345B",
    "catatan_pembayaran": "Pembayaran termin 1...",
    "bukti_pembayaran": "https://storage.smartelco.com/...",
    ...
  },
  "message": "Payment confirmed. Termin completed."
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

**🚀 Server:** `http://localhost:3000`  
**📅 Last Updated:** February 20, 2026
