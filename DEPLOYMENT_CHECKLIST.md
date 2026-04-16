# Railway Deployment Checklist ✅

## ⚠️ CRITICAL FIX - Application Crash on Startup

### Masalah:
Deploy logs hanya menunjukkan "Starting Container" tanpa log aplikasi = aplikasi crash sebelum sempat print log.

### Penyebab Umum:
1. Database connection gagal
2. Environment variables tidak diset
3. Binary compatibility issue
4. Port binding issue

### ✅ Yang Sudah Diperbaiki:
- Detailed logging di setiap step database connection
- Better error handling (tidak panic, tapi exit dengan error message)
- URL cleaning untuk SurrealDB client
- Auto-detect protocol (HTTP/HTTPS)

---

## Yang Harus Dilakukan di Railway Dashboard:

### 1. Set Environment Variables ⚠️ CRITICAL!

Buka: Railway Dashboard → reeng-be service → Variables

**PASTIKAN SEMUA VARIABLES INI ADA:**

```bash
# Database URL - Railway internal (tanpa protocol!)
SURREAL_URL=surrealdb.railway.internal:8080

# Database credentials
SURREAL_NAMESPACE=yerico
SURREAL_DATABASE=project_budget
SURREAL_USERNAME=root
SURREAL_PASSWORD=root

# JWT Secret
JWT_SECRET=token_admin@smartelco.com_1771569819

# Port (optional, Railway auto-set)
PORT=3001
```

**PENTING**: 
- Jangan tambahkan `http://` atau `https://` di SURREAL_URL
- Pastikan SurrealDB service sudah running di Railway
- Pastikan credentials match dengan SurrealDB service

### 2. Verify SurrealDB Service

Di Railway Dashboard:
1. Check apakah SurrealDB service status = "Active"
2. Check apakah ada error di SurrealDB logs
3. Pastikan SurrealDB menggunakan port 8080

### 3. Check Deploy Logs dengan Teliti

Railway Dashboard → Deployments → Latest → Deploy Logs

**Expected logs (urutan yang benar):**
```
Starting Container
🚀 Starting application...
SERVER_PORT: Ok("3001")
SURREAL_URL: Ok("surrealdb.railway.internal:8080")
🔧 Initializing AppState...
📋 Environment variables loaded:
   SURREAL_URL: surrealdb.railway.internal:8080
   SURREAL_NAMESPACE: yerico
   SURREAL_DATABASE: project_budget
🔌 Connecting to SurrealDB at surrealdb.railway.internal:8080...
🔗 Connecting with HTTP to: surrealdb.railway.internal:8080
✅ HTTP connection established
🔐 Signing in to database...
✅ Authentication successful
📂 Selecting namespace and database...
✅ Namespace and database selected
✅ Connected to SurrealDB: yerico/project_budget
✅ AppState initialized successfully
🚀 Server starting on http://0.0.0.0:3001
📝 Available endpoints:
  GET    /api/health
...
```

**Jika ada error, cari messages:**
- ❌ `SURREAL_URL environment variable not set` → Set di Railway Variables
- ❌ `Failed to establish HTTP connection` → Check SURREAL_URL format & SurrealDB status
- ❌ `Authentication failed` → Check SURREAL_USERNAME & SURREAL_PASSWORD
- ❌ `Failed to select namespace/database` → Check SURREAL_NAMESPACE & SURREAL_DATABASE

---

## Test Deployment:

```bash
# Health check
curl https://reeng-be-production.up.railway.app/api/health

# Expected response:
{
  "success": true,
  "message": "Server is running",
  "timestamp": "2026-04-15T...",
  "environment": {
    "surreal_url_set": true,
    "jwt_secret_set": true,
    "port": "3001"
  }
}
```

---

## Format SURREAL_URL yang Benar:

✅ **Correct:**
```
surrealdb.railway.internal:8080
localhost:8001
my-db.example.com:8080
```

❌ **Wrong:**
```
http://surrealdb.railway.internal:8080  (jangan tambahkan protocol)
https://surrealdb.railway.internal:8080 (jangan tambahkan protocol)
surrealdb.railway.internal              (harus include port)
```

Code akan otomatis menambahkan protocol yang sesuai:
- Railway internal (`railway.internal`) → HTTP
- Localhost/127.0.0.1 → HTTP
- External domain → HTTPS

---

## Troubleshooting Steps:

### Jika masih "Application failed to respond":

1. **Check Environment Variables**
   ```bash
   # Di Railway Dashboard → Variables
   # Pastikan SEMUA variables ada dan benar
   ```

2. **Check SurrealDB Service**
   ```bash
   # Di Railway Dashboard → SurrealDB service
   # Status harus "Active"
   # Check logs untuk error
   ```

3. **Check Deploy Logs**
   ```bash
   # Di Railway Dashboard → Deployments → View Logs
   # Cari error message terakhir sebelum crash
   ```

4. **Redeploy**
   ```bash
   # Setelah fix environment variables
   # Railway Dashboard → Deployments → Redeploy
   ```

5. **Check Network**
   ```bash
   # Pastikan reeng-be service bisa akses SurrealDB
   # Railway internal network harus configured
   ```

---

## Common Issues:

### Issue 1: "Starting Container" lalu tidak ada log
**Cause**: Aplikasi crash sebelum print log pertama
**Fix**: Check environment variables, pastikan semua ada

### Issue 2: "Failed to establish HTTP connection"
**Cause**: SURREAL_URL salah atau SurrealDB tidak running
**Fix**: 
- Check SURREAL_URL format (tanpa protocol, dengan port)
- Check SurrealDB service status

### Issue 3: "Authentication failed"
**Cause**: Username/password salah
**Fix**: Check SURREAL_USERNAME dan SURREAL_PASSWORD match dengan SurrealDB

### Issue 4: "Failed to select namespace/database"
**Cause**: Namespace/database tidak exist di SurrealDB
**Fix**: Create namespace/database di SurrealDB atau check spelling

---

## Next Steps After Successful Deploy:

1. Test health endpoint
2. Test login endpoint
3. Create initial admin user
4. Test other endpoints

Lihat `RAILWAY_DEPLOYMENT.md` untuk detail lengkap.
