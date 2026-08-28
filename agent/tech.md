# Technology Stack & Development Standards — Backend

Dokumen ini memuat daftar teknologi wajib, arsitektur dependensi, pustaka yang dilarang, serta strategi pengujian (*testing approach*) pada repo backend **CLI Adventure Sheet**.

---

## 1. Stack Teknologi Utama

Backend dibangun menggunakan ekosistem Rust modern berbasis async I/O:

| Komponen | Teknologi / Library | Versi | Fungsi / Alasan |
|---|---|---|---|
| **Bahasa & Edition** | Rust | Edition 2024 | Type-safety, performa tinggi, memory safety tanpa garbage collector. |
| **Web Framework** | Axum | `0.8.8` | Framework web modular & performan tinggi di atas Tokio & Tower. |
| **Async Runtime** | Tokio | `1.49.0` (feature: `full`) | Event loop & runtime I/O asynchronous standar ekosistem Rust. |
| **Database** | PostgreSQL | 15+ | Relational Database server terpusat. |
| **Query Builder / Driver** | SQLx | `0.8.6` | Pure Rust async SQL driver dengan compile-time checked queries. |
| **Authentication** | JWT (`jsonwebtoken`) | `10.3.0` | Stateless bearer token authentication (kadaluarsa 30 hari). |
| **Password Hashing** | Argon2 | `0.5.3` | Password hashing paling aman (Argon2id variant). |
| **Serialization** | Serde / Serde JSON | `1.0.228` / `1.0.149` | Serialisasi dan deserialisasi JSON request/response payload. |
| **Data Types** | UUID, Chrono, BigDecimal | `1.20`, `0.4`, `0.4` | Tipe data identifier V4, tanggal/waktu, dan presisi tinggi. |
| **Error Handling** | `thiserror` & `anyhow` | `2.0.18` & `1.0.101` | Strongly typed errors untuk HTTP responses & internal errors. |
| **Middleware & CORS** | Tower / Tower HTTP | `0.5.3` / `0.6.8` | Layer CORS & HTTP tracing logging via `tracing`. |

---

## 2. Migration Tooling

- **Tooling:** SQLx Migration Engine (`sqlx-cli` dan macro `sqlx::migrate!`).
- **Lokasi Files:** Berada di folder `/migrations` pada root repository backend.
- **Aturan Migration:**
  - Setiap perubahan skema database Wajib dibuat menggunakan SQL migration file berformat timestamped (contoh: `20260220000000_create_characters_table.sql`).
  - Migration **HARUS bersifat reversibel** atau terisolasi dengan aman.
  - Penambahan kolom baru pada tabel yang sudah ada disarankan memiliki default value atau nullable agar tidak merusak data lama.
  - Jalankan `sqlx migrate run` saat local development atau ci/cd deployment.

---

## 3. Pustaka & Praktik yang DILARANG (Forbidden List)

| Hal yang Dilarang | Alasan & Risiko | Solusi / Alternatif Wajib |
|---|---|---|
| **ORM Lain (Diesel, SeaORM)** | Menyebabkan duplikasi connection pool, bloat dependensi, dan konflik abstraksi dengan SQLx yang sudah terpilih. | Gunakan **SQLx** secara penuh untuk query layer. |
| **Async Runtime Lain (async-std, smol)** | Menyebabkan panic runtime mutlak karena Axum & SQLx terikat erat pada `tokio`. | Gunakan **Tokio** runtime. |
| **String Interpolation SQL (`format!("SELECT ... {}", val)`)** | **Kerentanan Keamanan Kritis SQL Injection.** | Wajib gunakan **Parameterized Queries** (`$1, $2` via `sqlx::query!` atau `.bind()`). |
| **Algoritma Hash Lemah (MD5, SHA1, Bcrypt lama)** | Rentan terhadap attack GPU cracking modern. | Wajib gunakan **Argon2id** (`argon2` crate). |
| **Kalkulasi Aturan D&D di Frontend** | Merusak prinsip *authoritative backend* dan berisiko terjadinya manipulasi stat karakter dari client. | Seluruh kalkulasi stat & validasi aturan **HARUS dilakukan di Backend Rust**. |
| **Unwrapped Raw Database Error Response** | Membocorkan nama kolom DB, struktur tabel, dan detail internal ke client. | Tangkap error di layer `error.rs` dan ubah menjadi pesan HTTP berstruktur standar. |

---

## 4. Kontrak API & Dokumentasi (`API_DOCUMENTATION.md`)

- **Sumber Kebenaran (Single Source of Truth):** File `API_DOCUMENTATION.md` di root repo backend ini adalah dokumen resmi yang memuat seluruh spesifikasi endpoint, HTTP method, path, request/response shape, HTTP status code, dan format error. File ini dirujuk oleh repo frontend (disalin secara manual setiap ada update) untuk mendesain dan membangun `crates/core` di sisi frontend.
- **Proses Kerja Wajib:** Setiap kali ada penambahan endpoint baru **ATAU** perubahan struktur/shape pada request body/response body dari endpoint yang sudah ada, Pull Request (PR) atau commit yang bersangkutan **WAJIB** menyertakan pembaruan pada `API_DOCUMENTATION.md` dalam **commit yang sama**. Dilarang keras membuat commit/PR terpisah yang "menyusul nanti".
- **Dampak Multi-Target Frontend:** Karena API backend ini dikonsumsi oleh 4 target frontend sekaligus (CLI, Desktop, Web, dan Mobile), kelalaian atau drift dalam memperbarui dokumentasi akan berdampak fatal pada keempat target frontend tersebut secara bersamaan.

---

## 5. Strategi Pengujian (Testing Approach)

Pengujian dibagi menjadi dua tingkatan utama:

### A. Unit Testing (`src/models/` & `src/services/`)
- **Fokus:** Menguji logika bisnis murni tanpa koneksi database.
- **Cakupan:**
  - Kalkulasi Ability Modifier (`floor((score - 10) / 2)`).
  - Kalkulasi Armor Class & Hit Points.
  - Evaluasi prerequisite Feat dan ASI.
  - Penentuan status slot Progression Manifest (`completed`, `partial`, `pending`, `locked`).
- **Lokasi:** Inline unit tests di bagian bawah file Rust masing-masing (`#[cfg(test)] mod tests { ... }`).

### B. Integration Testing (`tests/`)
- **Fokus:** Menguji alur HTTP API endpoint dari request hingga response database.
- **Tools:** `tokio::test`, `tower::ServiceExt`, `http-body-util`, `sqlx::test`.
- **Cakupan:**
  - Endpoint authentication (`/signup`, `/login`, token invalid).
  - CRUD Karakter & penegakan ownership (`user_id`).
  - Validation failures (misal: submit ASI illegal mengembalikan HTTP 400).
  - Istirahat & pemakaian resource (`/short-rest`, `/long-rest`).
