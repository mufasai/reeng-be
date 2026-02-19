# Simple API - SmartElco Backend

Backend API untuk SmartElco Reengineering Tracking Tool yang telah dirapikan dengan struktur modular.

## 📁 Struktur Folder

```
simple-api/
├── Cargo.toml                 # Rust dependencies
├── .env                       # Environment variables
├── README.md                  # This file
└── src/
    ├── main.rs               # Main application & router setup
    ├── models.rs             # Data models & structs
    ├── state.rs              # Application state management
    └── handlers/
        ├── mod.rs            # Module exports
        ├── auth.rs           # Authentication handlers (login)
        └── project.rs        # Project handlers (create, list)
```

## 🎯 Penjelasan Struktur

### `src/main.rs`
- Entry point aplikasi
- Setup router dengan semua endpoints
- CORS configuration
- Server initialization

### `src/models.rs`
Berisi semua data structures:
- `LoginRequest`, `LoginResponse`, `UserInfo` - Auth models
- `Project`, `ProjectSite`, `PersonnelInfo` - Project models
- `ProjectType` - Enum untuk tipe project
- `ApiResponse<T>` - Generic response wrapper

### `src/state.rs`
- `AppState` - Application state
- In-memory storage untuk projects (untuk development)

### `src/handlers/auth.rs`
- `login()` - Handle POST /api/auth/login
- Simple authentication dengan hardcoded credentials

### `src/handlers/project.rs`
- `create_project()` - Handle POST /api/projects
- `list_projects()` - Handle GET /api/projects

## 🚀 Running Backend

```bash
# Dari root tracking-tool folder
cd simple-api
cargo run
```

Backend akan running di **http://localhost:3000**

## 📋 Available Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check |
| POST | `/api/auth/login` | Login with email/password |
| POST | `/api/projects` | Create new project with sites |
| GET | `/api/projects` | List all projects |

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
