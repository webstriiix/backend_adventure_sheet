# Tech Stack & Aturan Engineer

## Technology Stack
- **Bahasa**: Rust (Edition 2024)
- **Web Framework**: Axum (v0.8.x) bersama `tokio` untuk runtime asynchrony.
- **Database**: PostgreSQL
- **Query Builder/ORM**: SQLx (v0.8.x) untuk eksekusi PostgreSQL *type-safe*.
- **Auth**: JWT (via `jsonwebtoken`) + Argon2 untuk password hashing.

## Workflow Migration (SQLx)
Semua skema database tertuang dalam folder `migrations/`. Terdapat dua cara mengeksekusi migration:
- **`sqlx migrate run`** (direkomendasikan).
- Manual query via DB shell `psql`.

> **CATATAN INSIDEN (Checksum Mismatch)**: Jika suatu migration dijalankan secara manual (lewat `psql`), backend akan panik (panicking on startup) akibat `checksum mismatch` yang dilacak `sqlx`. Untuk mencegah ini, **sangat disarankan selalu menggunakan `sqlx migrate run` dari awal**. Jika terpaksa jalan manual, checksum di dalam tabel database `_sqlx_migrations` WAJIB diperbarui atau di-sinkronkan agar sesuai dengan hash migrasi SQL lokal yang sebenarnya.

## Local Development (Docker Compose)
Kebutuhan database pengembangan (dev database) digulirkan melalui `Docker Compose`:
- File terdefinisi di `compose.yml` pada root repository.
- Menyediakan PostgreSQL di `localhost:5432`.
- Untuk username/pass standar, rujuk variabel lingkungan di `.env.example` — DILARANG mencantumkan/men-commit file konfigurasi yang memuat kredensial absolut.

## Kontrak API & Dokumentasi
File **`API_DOCUMENTATION.md`** di *root* repository adalah satu-satunya **sumber kebenaran abstrak (source of truth)** untuk kontrak endpoint antar backend dan 4 frontend client. 
- API doc ini disalin secara manual (copy/paste sync) ke repository frontend untuk referensi.
- **ATURAN WAJIB**: Setiap perubahan, penambahan, penghapusan *endpoint*, *header*, atau *schema payload JSON* **HARUS** di-update secara simultan di `API_DOCUMENTATION.md` dalam *Pull Request* yang sama bersamaan dengan perubahan kode.

## Pelarangan Library
- Jangan menambah library besar berbasis sinkron yang akan mem-block runtime `tokio`. Segala bentuk HTTP fetching (jika ada) wajib dari client async (seperti `reqwest`), bukan tipe sync.
- Jangan menggunakan ORM besar (seperti Diesel/SeaORM) karena basis projek dikhususkan dan distandardisasi dengan **SQLx**.

## Testing Requirements
Terdapat sebuah skrip otomatis di `tests/smoke_test.py`. Skrip python ini adalah **automated smoke test** khusus yang memverifikasi database constraints. **Test ini WAJIB dijalankan setelah reset/re-import keseluruhan data** (bukan sekedar spot check manual parsial) sebelum disimpulkan aman.
