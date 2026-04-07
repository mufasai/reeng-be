## Valid Enum / Stage Value

Berdasarkan konfigurasi backend (`src/config.rs`), berikut nilai String *exact* yang dikirim pada field `stage`:

### Workflow Default (13 Tahapan)
Untuk project biasa (Beban Operasional, dll):
1. `imported`
2. `assigned`
3. `permit_process`
4. `permit_ready`
5. `akses_process`
6. `akses_ready`
7. `implementasi`
8. `rfi_done`
9. `rfs_done`
10. `dokumen_done`
11. `bast`
12. `invoice`
13. `completed`

### Workflow Khusus "RESCOPING" (17 Tahapan)
Khusus Site bertipe "RESCOPING", terdapat tahap tambahan untuk skenario gagal survei dan ERFIN:
1. `imported`
2. `assigned`
3. `survey` *(Tambahan)*
4. `survey_nok` *(Tambahan)*
5. `erfin_process` *(Tambahan)*
6. `erfin_ready` *(Tambahan)*
7. `permit_process`
8. `permit_ready`
9. `akses_process`
10. `akses_ready`
11. `implementasi`
12. `rfi_done`
13. `rfs_done`
14. `dokumen_done`
15. `bast`
16. `invoice`
17. `completed`

**Rule Transisi Backend:**
- Update API **tidak mengizinkan** nilai sembarangan.
- Maksimal +1 Stage (contoh: `imported` hanya bisa ke `assigned`).
- **Mundur (Backward)** tidak diperbolehkan secara API.
- Apabila ada field wajib kosong saat *next stage* dikonfirmasi (contoh: *survey date* kosong saat mencoba masuk tahap *survey*), API me-return `400 Bad Request`.