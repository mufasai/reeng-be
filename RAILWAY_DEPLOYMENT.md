# Railway Deployment Troubleshooting

## Error 502: Application Failed to Respond

### Penyebab Umum:

1. **Environment Variables Tidak Diset**
   - Railway membutuhkan environment variables yang diset di dashboard
   - File `.env` lokal TIDAK otomatis ter-upload ke Railway

2. **Database Connection Gagal**
   - `SURREAL_URL=localhost:8001` tidak akan berfungsi di Railway
   - Perlu URL database yang bisa diakses dari Railway

3. **Port Configuration**
   - Railway menggunakan environment variable `PORT` (bukan `SERVER_PORT`)
   - Aplikasi sudah diperbaiki untuk membaca `PORT` terlebih dahulu

---

## Solusi:

### 1. Set Environment Variables di Railway Dashboard

Buka Railway Dashboard → Service → Variables, lalu tambahkan:

```bash
# Port (Railway akan set otomatis, tapi bisa override)
PORT=3001

# JWT Secret
JWT_SECRET=token_admin@smartelco.com_1771569819

# SurrealDB Configuration
# PENTING: Ganti dengan URL database yang benar!
SURREAL_URL=https://your-surrealdb-instance.com
# atau jika menggunakan Railway SurrealDB service:
# SURREAL_URL=${{SURREALDB.RAILWAY_PRIVATE_DOMAIN}}:8000

SURREAL_NAMESPACE=yerico
SURREAL_DATABASE=project_budget
SURREAL_USERNAME=root
SURREAL_PASSWORD=root
```

### 2. Setup SurrealDB di Railway

Ada 2 opsi:

#### Opsi A: Deploy SurrealDB di Railway (Recommended)
1. Klik "New" → "Database" → "SurrealDB"
2. Setelah deploy, copy connection URL
3. Set `SURREAL_URL` ke URL tersebut

#### Opsi B: Gunakan External SurrealDB
1. Pastikan SurrealDB bisa diakses dari internet
2. Set `SURREAL_URL` ke URL public database

### 3. Redeploy Aplikasi

Setelah set environment variables:
1. Commit & push perubahan code terbaru
2. Railway akan auto-redeploy
3. Check logs: Railway Dashboard → Deployments → View Logs

---

## Cara Check Logs di Railway:

1. Buka Railway Dashboard
2. Pilih service "reeng-be"
3. Klik tab "Deployments"
4. Klik deployment terakhir
5. Klik "View Logs"

Cari error messages seperti:
- `SURREAL_URL environment variable not set`
- `Connection refused`
- `Failed to connect to database`

---

## Testing Setelah Deploy:

```bash
# Health check
curl https://reeng-be-production.up.railway.app/api/health

# Expected response:
{
  "success": true,
  "message": "Server is running",
  "timestamp": "2026-04-15T..."
}
```

---

## Perubahan yang Sudah Dilakukan:

1. ✅ Port configuration diperbaiki untuk membaca `PORT` env variable
2. ✅ Error handling ditambahkan untuk environment variables
3. ✅ Logging ditambahkan untuk debugging

## Next Steps:

1. Set environment variables di Railway Dashboard
2. Setup SurrealDB connection
3. Redeploy dan check logs
