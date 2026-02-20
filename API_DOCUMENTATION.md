# 📚 Reengineering Tool Backend - API Documentation

**Base URL:** `http://localhost:3000/api`

---

## 🔐 Authentication

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

**Request Body:**
```json
{
  "project_id": "projects:b7v5e43bvtpwyipxlemg",
  "site_id": "sites:73tnamhln5s1oehr2om2",
  "type_termin": "Termin 1 - 30%",
  "tgl_terima": "2026-02-20",
  "jumlah": 150000000,
  "status": "pending",
  "keterangan": "Pembayaran termin pertama setelah pekerjaan 30% selesai"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "termins:term001",
    "project_id": "projects:b7v5e43bvtpwyipxlemg",
    "site_id": "sites:73tnamhln5s1oehr2om2",
    "type_termin": "Termin 1 - 30%",
    "tgl_terima": "2026-02-20",
    "jumlah": 150000000,
    "status": "pending",
    "keterangan": "Pembayaran termin pertama...",
    "created_at": "2026-02-20T10:00:00Z"
  },
  "message": "Termin created successfully"
}
```

### List All Termins
**GET** `/termins`

### Get Termins by Project
**GET** `/termins/project/:project_id`

### Get Termins by Site
**GET** `/termins/site/:site_id`

### Upload Termin File
**POST** `/termin-files`

**Request Body:**
```json
{
  "termin_id": "termins:term001",
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
