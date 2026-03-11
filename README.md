# Reengineering Tool Backend

Backend API untuk Reengineering Tracking Tool dengan sistem manajemen Project, Site, Team, People, Costs, Materials, dan Termin.

## ✨ Key Features

- 🔐 **Authentication & User Management** - JWT-based auth dengan role-based access control
- 📊 **Excel Bulk Import** - Import Project + Sites dari Excel dengan smart type detection (FILTER, COMBAT, L2H, etc.)
- 🏷️ **Auto Project Type Detection** - Project type & name auto-extracted dari Column B (TIPE PROJECT)
- 📁 **Project Management** - Full CRUD untuk Projects dengan keterangan, budget tracking, dates
- 🏗️ **Site Management** - Site creation dengan team assignment, lokasi, budget, kontrak
- 👥 **Team & People** - Team creation dengan leader & members, personnel tracking
- 💰 **Cost & Material Tracking** - Track biaya dan material per site
- 📅 **Termin Management** - 4-stage payment system (30-50-10-10) dengan flexible amounts
- 📎 **File Upload & Storage** - Multipart file uploads dengan base64 storage di database
- 📥 **File Download** - Download bukti pembayaran, site documents, project files

## 📁 Struktur Folder

```
reengineering-tool-be/
├── Cargo.toml                 # Rust dependencies
├── .env                       # Environment variables
├── README.md                  # This file
└── src/
    ├── main.rs                         # Main application & router setup
    ├── models.rs                       # Data models & structs (15+ models)
    ├── state.rs                        # Application state management (SurrealDB)
    └── handlers/
        ├── mod.rs                      # Module exports
        ├── auth.rs                     # Authentication & user management
        ├── project.rs                  # Project CRUD handlers
        ├── site.rs                     # Site CRUD handlers (with team creation)
        ├── people.rs                   # People CRUD handlers
        ├── costs.rs                    # Cost tracking handlers
        ├── materials.rs                # Material tracking handlers
        ├── termins.rs                  # Termin (payment stages) handlers
        ├── regions.rs                  # Region management handlers
        ├── files.rs                    # File upload/download handlers
        └── bulk_import.rs              # Excel bulk import handler (NEW!)
```

## 🎯 Penjelasan Struktur

### `src/main.rs`
- Entry point aplikasi
- Setup router dengan 50+ endpoints
- CORS configuration untuk cross-origin requests
- SurrealDB client initialization
- Server initialization dengan Tokio runtime

### `src/models.rs`
Berisi 20+ data structures:
- **Auth:** `LoginRequest`, `RegisterRequest`, `User`, `UserRole`
- **Projects:** `Project`, `CreateProjectRequest`, `UpdateProjectRequest`, `ProjectType`
- **Sites:** `Site`, `CreateSiteRequest`, `UpdateSiteRequest`
- **Teams:** `Team`, `TeamPeople`, `TeamMemberWithInfo`
- **People:** `Person`, `CreatePersonRequest`
- **Costs & Materials:** `Cost`, `Material`
- **Termins:** `Termin`, `PayTerminRequest`, `TerminWithSiteInfo`
- **Files:** `ProjectFile`, `SiteFile`, `TerminFile`
- **Bulk Import:** `BulkImportExcelResponse`, `ImportError`, `ImportSummary` ⭐ NEW
- **Generic:** `ApiResponse<T>` - Response wrapper

### `src/state.rs`
- `AppState` - Application state dengan SurrealDB client
- Database connection pooling
- Shared state untuk semua handlers

### `src/handlers/bulk_import.rs` ⭐ NEW
- **Excel Parsing:** Parse .xlsx files dengan `calamine` crate
- **Sheet 3 Targeting:** Extract "Active Project Details" sheet
- **Smart Extraction:** Filename parsing untuk project name & date
- **Row 2 Totals:** Extract project budget dari summary row
- **Column Mapping:** 15+ Excel columns → Site model fields
- **Atomic Creation:** Create 1 Project + N Sites (tested with 36 sites)
- **Error Resilience:** Continue processing dengan per-row error collection
- **Auto-generation:** Missing fields auto-filled dengan defaults

### Other Handlers
- `auth.rs` - Register, Login, User CRUD
- `project.rs` - Project CRUD, file upload
- `site.rs` - Site CRUD dengan auto team creation
- `termins.rs` - 4-stage payment system dengan file upload/download
- `files.rs` - Generic file upload/download untuk berbagai entities

## 🚀 Running Backend

```bash
# 1. Install SurrealDB (jika belum)
# macOS:
brew install surrealdb/tap/surreal

# 2. Start SurrealDB
surreal start --bind 0.0.0.0:8001 --user root --pass root memory

# 3. Setup .env file
cp .env.example .env
# Edit .env sesuai konfigurasi

# 4. Run backend
cd reengineering-tool-be
cargo run
```

Backend akan running di **http://localhost:3000**

## 📋 Available Endpoints

### 🔐 Authentication
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/auth/register` | Register new user dengan role |
| POST | `/api/auth/login` | Login dengan email/password |
| GET | `/api/users` | List all users |
| GET | `/api/users/:id` | Get user by ID |
| PUT | `/api/users/:id` | Update user |
| DELETE | `/api/users/:id` | Delete user |

### 📁 Projects
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/projects` | Create new project |
| GET | `/api/projects` | List all projects |
| GET | `/api/projects/:id` | Get project by ID |
| PUT | `/api/projects/:id` | Update project |
| DELETE | `/api/projects/:id` | Delete project |
| **POST** | **`/api/projects/import-excel`** | **⭐ Bulk import dari Excel** |
| POST | `/api/projects/:id/upload` | Upload project file |
| GET | `/api/project-files/:id/download` | Download project file |

### 🏗️ Sites
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/sites` | Create site (with team) |
| GET | `/api/sites` | List all sites |
| GET | `/api/sites/project/:id` | Get sites by project |
| GET | `/api/sites/:id` | Get site by ID |
| PUT | `/api/sites/:id` | Update site |
| DELETE | `/api/sites/:id` | Delete site |
| POST | `/api/sites/:id/upload` | Upload site file |
| GET | `/api/site-files/:id/download` | Download site file |

### 👥 Teams & People
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/teams` | Create team dengan members |
| GET | `/api/teams` | List all teams |
| GET | `/api/teams/:id` | Get team detail |
| GET | `/api/teams/:id/members` | Get team members |
| GET | `/api/teams/leader/:leader_id` | Get team by leader |
| POST | `/api/people` | Create person |
| GET | `/api/people` | List all people |

### 💰 Costs & Materials
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/costs` | Add cost to site |
| GET | `/api/costs/site/:site_id` | Get costs by site |
| POST | `/api/materials` | Add material to site |
| GET | `/api/materials/site/:site_id` | Get materials by site |

### 📅 Termins (Payment Stages) 
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/termins` | Create termin untuk site |
| GET | `/api/termins` | List all termins |
| GET | `/api/termins/project/:id` | Get termins by project |
| GET | `/api/termins/site/:id` | Get termins by site |
| POST | `/api/termins/:id/pay` | Pay termin (JSON or multipart) |
| GET | `/api/termins/:id/download-bukti-pembayaran` | Download payment proof |

### 🌍 Regions
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/regions` | Create region |
| GET | `/api/regions` | List all regions |

### ✅ Health Check
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check & timestamp |

## 🔥 Excel Bulk Import Feature

### Quick Start
```bash
# Upload Excel file untuk create Project + Sites
curl -X POST http://localhost:3000/api/projects/import-excel \
  -F 'file=@EPROC20260206001_SST_BOQ_IRR_Filter_Batch_5_and_4_R12_Eastern_Jakarta.xlsx'
```

### Excel Format Requirements
- **Sheet:** "Active Project Details" (Sheet 3)
- **Filename (Flexible):** 
  - `EPROC{DATE}_{Details}_{Type}_{Location}.xlsx` (e.g., Filter, Combat, L2H)
  - `OSP Project Report_Update-YYYYMMDD-LOCATION.xlsx`
- **Row 2:** Total values (BOQ AKTUAL @ column I, TOTAL NILAI PO @ column M)
- **Row 5:** Column headers (including **TIPE PROJECT** at column B)
- **Row 6:** First data row - **Column B determines project type & name**
- **Row 6+:** Site data (100+ rows supported)

### Project Auto-Generation ⭐ NEW
**Project name format:** `{TIPE} Project {LOKASI}`

Examples based on Column B (TIPE PROJECT):
- Column B = "FILTER" → Project name: **"FILTER Project Jakarta"**
- Column B = "COMBAT" → Project name: **"COMBAT Project Surabaya"**
- Column B = "L2H" → Project name: **"L2H Project Bandung"**

**Supported project types:**
- COMBAT, L2H, BLACK SITE, REFINEN, FILTER, BEBAN OPERASIONAL

**Auto-extracted from Excel:**
- ✅ Project type: From Column B (TIPE PROJECT) Row 6
- ✅ Project name: `{TIPE} Project {LOKASI}`
- ✅ Location: Last part of filename (e.g., "Jakarta", "Jabo", "PEKALONGAN")
- ✅ Date: First 8-digit number in filename (YYYYMMDD)
- ✅ Budget: Row 2 totals (Column I & M)

### Column Mapping (15+ fields)
| Excel Column | Index | Maps To | Type | Notes |
|--------------|-------|---------|------|-------|
| **B: TIPE PROJECT** | **1** | **`project.tipe`** | **enum** | **⭐ Determines project type & name** |
| L: NAMA LOP [SITE] | 11 | `site_name` | string | required |
| D: WTIEL | 3 | `lokasi` | string | |
| K: NAMA PO | 10 | `pekerjaan` | string | |
| J: NOMOR PO | 9 | `nomor_kontrak` | string | auto-gen if empty |
| G: TANGGAL WO | 6 | `start` | date | |
| O: TANGGAL | 14 | `end` | date | fallback to start |
| M: NILAI PO | 12 | `maximal_budget` | i64 | |
| H: BOQ KONTRAK | 7 | `cost_estimated` | i64 | |
| B+N+P (combined) | - | `site_info` | string | includes type, status, notes |

### Response Example
```json
{
  "success": true,
  "data": {
    "project": { 
      "id": "projects:xxx", 
      "name": "FILTER Project Jakarta", 
      "tipe": "FILTER",
      ... 
    },
    "total_rows": 36,
    "sites_created": 36,
    "sites_failed": 0,
    "created_sites": [ /* array of 36 sites */ ],
    "errors": [],
    "summary": {
      "project_id": "projects:xxx",
      "project_name": "FILTER Project Jakarta",
      "total_budget": 257091760,
      "sites_count": 36,
      "message": "Import completed: 36 sites created, 0 failed out of 36 rows"
    }
  }
}
```

### Benefits
- ⚡ **Fast:** Import 100+ sites in seconds (vs hours manual entry)
- 🔄 **Atomic:** All-or-nothing per site (continues on error)
- 📊 **Detailed:** Per-row error reporting with field-level info
- 🔗 **Relational:** Auto-links all sites to project via project_id
- 🎯 **Smart:** Auto-extract project type, name, location, date from Excel
- 🏷️ **Type-aware:** Supports 6 project types (FILTER, COMBAT, L2H, etc.)
- 📝 **Flexible:** Multiple filename formats supported
- 🛡️ **Resilient:** Skips empty rows, auto-generates missing fields

## 🔧 Development

### Add New Endpoint

1. **Create handler function** di folder `handlers/`
   ```rust
   // handlers/your_module.rs
   pub async fn your_handler() -> Result<...> {
       // handler logic
   }
   ```

2. **Export di `handlers/mod.rs`**
   ```rust
   pub mod your_module;
   ```

3. **Add route di `main.rs`**
   ```rust
   .route("/api/your-path", get(your_module::your_handler))
   ```

### Add New Model

Tambahkan struct di `src/models.rs`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YourModel {
    pub field: String,
}
```

## ✅ Testing

```bash
# Health check
curl http://localhost:3000/api/health

# Login
curl -X POST http://localhost:3000/api/auth/login \\
  -H "Content-Type: application/json" \\
  -d '{"email":"admin@smartelco.com","password":"admin123"}'

# Create project
curl -X POST http://localhost:3000/api/projects \\
  -H "Content-Type: application/json" \\
  --data @../test-project.json

# List projects
curl http://localhost:3000/api/projects
```

## 📦 Dependencies

Lihat `Cargo.toml` untuk daftar lengkap. Main dependencies:
- **axum** - Web framework
- **tokio** - Async runtime
- **serde** - Serialization/deserialization
- **tower-http** - CORS middleware
- **uuid** - ID generation
- **chrono** - Date/time handling

## 🎨 Code Style

- **Handlers**: Async functions yang menerima request dan return response
- **Models**: Pure data structures dengan derives untuk Serialize/Deserialize
- **State**: Shared application state dengan Arc<Mutex<>> untuk thread-safety
- **Error handling**: Result types dengan proper error responses

## 🔄 Migration dari Monolithic

File sebelumnya (`main.rs` dengan 250+ lines) telah dipecah menjadi:
- ✅ Handlers dipisah per module (auth, project)
- ✅ Models dipisah ke file terpisah
- ✅ State management terpisah
- ✅ Main.rs hanya fokus pada routing & setup

## 🎓 Best Practices

1. **Separation of Concerns** - Setiap module punya tanggung jawab jelas
2. **Modularity** - Mudah add/remove features
3. **Maintainability** - Code lebih mudah dibaca dan di-maintain
4. **Scalability** - Struktur siap untuk project yang lebih besar

---

**Status**: ✅ Running & Tested
**Port**: 3000
**Environment**: Development
