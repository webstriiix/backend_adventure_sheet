# Arsitektur & Peta Kode Backend

Folder Backend tersusun sedemikian rupa dengan peran dan tanggung jawab direktori terpisah.

## Peta Direktori & Lokasi Signifikan

- **`src/main.rs`** 
  Konfigurasi port environment, setup HTTP layer global (Core rate limiting, CORS configuration, payload limits), dan router mount point.
- **`src/routes/`**
  (`src/routes/mod.rs`) Mengandung routing tree Axum (`Router::new().route(...)`), menggabungkan *handler* untuk base URL `/api/v1/*`.
- **`src/handlers/`**
  Controller endpoint HTTP. Semua permintaan ditarik ke dalam file di sini, dibagi per resource (misalnya `auth.rs`, `compendium.rs`). Penanganan karakter D&D diisolasi secara mendalam di folder `src/handlers/characters/` karena memiliki sub-logic kompleks seperti aksi `rests.rs`, `spells.rs`, dan `progression.rs`.
- **`src/models/`**
  Rust Structs (dengan macro `serde` JSON derive). Memodelkan tabel psql 1:1 map (`class.rs`, `items.rs`, `monsters.rs`, dsb.).
- **`src/importers/`**
  Pusat parsing dan ekstraksi JSON format 5etools ke format insersi SQLx.
  **Lokasi FIle Kunci (Bukti Historis)**: `src/importers/import_classes.rs` dan file serupa adalah titik paling rentan *(prone area)* terhadap inkonsistensi struktur versi source. Proses ini mencakup manipulasi parsing ON CONFLICT (Baca historis investigasi debug pada `importers` sebelum memodifikasinya).
- **`src/services/`**
  (`src/services/auth.rs`) Modul untuk pemisahan logika enkripsi, bisnis verifikasi JWT dan password.
- **`migrations/`**
  Repoting migrasi SQL mentah yang di-*track* SQLx.
- **`tests/`**
  Skrip pengujian integrasi (`smoke_test.py`) dan automated unit integration tests Rust (`wizard_integration_tests.rs`).

## Aturan Penempatan Perubahan Fitur Baru

Jika sebuah fitur / task baru masuk:
1. **Tabel Basis Data Baru / Perubahan Schema**:
   Buat migrasi SQL baru di `migrations/` dengan urutan timestamp waktu standar SQLx.
2. **Representasi Bentuk Struct & Field Map**:
   Buat struct Rust baru (atau edit struct rujukan) di dalam `src/models/`.
3. **Logika Endpoint & Rule Karakter**:
   Buat endpoint di `src/handlers/`, pisahkan file sub-logika ke `src/handlers/characters/` jika spesifik mengendalikan sheet pemain.
4. **Logika Importer (Jika bersumber dari 5eTools JSON)**:
   Buat atau ubah modul di `src/importers/`. Pastikan logic filter _transitional reference_ berlaku aman.
5. **Kontrak Eksternal**:
   Registrasikan rute di `src/routes/mod.rs` & wajib dokumentasikan payloads di `API_DOCUMENTATION.md` (akar repositori).
