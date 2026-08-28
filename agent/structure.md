# Peta Struktur Repository — Backend

Repository ini menggunakan pola *Service-Oriented Architecture* untuk memisahkan domain logic (aturan D&D 5e) dari layer transport (Axum handlers) dan layer persisten (SQLx DB).

## 1. Direktori Utama

```text
adventure_sheets/
├── agent/            # Dokumentasi & Panduan Agent
├── migrations/       # SQL Migration files (versioned)
└── src/
    ├── handlers/     # Axum endpoint wrappers (request context parsing)
    ├── models/       # Struct domain (Database model & DTOs)
    ├── routes/       # Axum Route definitions
    ├── services/     # Rule validation & kalkulasi bisnis
    ├── importers/    # Logika import data kompendium (5etools format)
    └── db.rs         # Database connection pool state
```

## 2. Aturan Penempatan Kode

| Jenis Perubahan | Penempatan File | Keterangan |
|---|---|---|
| **Resource Baru (API)** | `src/routes/` | Tambah module route baru, daftarkan di `main.rs`. |
| **Logic Endpoint** | `src/handlers/{resource}/` | Parsing payload, Auth check (user_id), panggil service. |
| **Aturan Game (5e)** | `src/services/` | Logika validasi dan kalkulasi stats final. |
| **Domain Data** | `src/models/` | Struct representasi database atau DTO. |

---

## 3. Contoh Path Konkret

- **`src/models/character.rs`**: Struct `Character` dan impl methods dasar.
- **`src/handlers/characters/progression.rs`**: Endpoint API untuk `GET /characters/{id}/progression`, parse request, panggil `manifest_service`.
- **`src/services/manifest_service.rs`**: Logic utama evaluasi progression manifest (tidak ada di repo, harus dibuat).
- **`src/routes/mod.rs`**: Pendaftaran endpoint character: `.route("/characters/:id/progression", get(handlers::characters::progression::get_manifest))`.

---

## 4. Alur Tambah Fitur Baru (Best Practice)

1. **Definisikan Schema DB** (jika butuh storage baru): Buat file di `migrations/`.
2. **Definisikan Domain Model**: Tambah struct di `src/models/`.
3. **Impelementasi Rule Service**: Buat logika di `src/services/` (lakukan kalkulasi/validasi di sini, bukan di handler).
4. **Implementasi Handler**: Parsing input di `src/handlers/`, panggil service.
5. **Daftarkan Route**: Tambah route di `src/routes/`.
6. **Update Dokumentasi API**: Wajib perbarui `API_DOCUMENTATION.md` di root repo dalam commit yang sama.
