# Aturan Pengembangan & Keamanan Backend

Dokumen ini mendefinisikan standar pengkodean dan kebijakan keamanan mandatori untuk seluruh kontribusi pada backend **CLI Adventure Sheet**.

---

## 1. Aturan Validasi & Keamanan

### A. Parameterized Queries mandatory
- **DILARANG KERAS** menggunakan manipulasi string/format untuk membuat query SQL:
  ```rust
  // DILARANG
  sqlx::query(&format!("SELECT * FROM items WHERE name = '{}'", user_input))
  ```
- **WAJIB** gunakan parameter SQLx binding:
  ```rust
  // WAJIB
  sqlx::query!("SELECT * FROM items WHERE name = $1", user_input)
  ```

### B. Input Validation
- Seluruh endpoint publik yang menerima input body (`POST`/`PUT`/`PATCH`/`POST`) **WAJIB** memvalidasi setiap field di `services/` layer (menggunakan crate `validator` jika perlu, atau logic manual).
- Backend tidak pernah mempercayai data dari klien sebagai kebenaran mutlak.

### C. Versioning API
- Seluruh endpoint API publik **WAJIB** memiliki prefix `/api/v1/`.
- Jangan melakukan *breaking changes* pada endpoint tanpa menyediakan route versi `/api/v2/` atau memperbarui dokumentasi API secara eksplisit.

### D. Penanganan Auth
- Setiap endpoint yang mengubah data (selain login/signup/compendium view) **WAJIB** mengekstrak dan memvalidasi `user_id` dari JWT Bearer Token, dan memverifikasi bahwa `character_id` yang diakses benar-benar milik user tersebut (gunakan `verify_character_ownership` helper).

---

## 2. Larangan Kalkulasi Duplikasi

1. **Keep Consistency:** Jangan lakukan perhitungan ability modifier atau HP di luar `services/` atau `models/` impl.
2. **Single Truth Source Service:** Jika ada logika yang melibatkan *Progression Manifest* (penghitungan level, penentuan status slot), semua *write* ke database terkait manifest **HARUS** diproses melalui satu `progression_service` terpusat untuk menghindari duplikasi logic.
3. **No Database Leakage:** Jangan pernah mengirim error database mentah ke client (`?` operator pada SQLx harus di-map ke `crate::error::AppError` yang sudah disanitasi).

---

## 4. Keamanan Data sensitif
- Hashing password: Hanya boleh menggunakan **Argon2id**.
- Penyimpanan token: JWT di-sign dengan `secret` yang dimuat via env variable (`dotenvy`).

---

## 5. Sinkronisasi Kontrak API (`API_DOCUMENTATION.md`)

### A. Larangan Drift Kontrak
SETIAP endpoint baru atau perubahan pada request/response shape endpoint yang sudah ada WAJIB disertai update pada `API_DOCUMENTATION.md` dalam PR/commit yang sama. JANGAN PERNAH merge perubahan API tanpa update dokumentasi ini — hal tersebut akan menyebabkan drift kontrak dengan `crates/core` di sisi frontend, yang berdampak ke 4 target sekaligus (CLI/Desktop/Web/Mobile) tanpa ada error yang jelas di sisi backend sendiri (kegagalan baru terlihat saat frontend mencoba deserialize response).

### B. Aturan Format Entry `API_DOCUMENTATION.md`
Setiap entri penulisan endpoint baru di `API_DOCUMENTATION.md` wajib mengikuti struktur standar agar dapat disalin secara langsung ke modul frontend:
1. **Method & Path** (misal: `POST /api/v1/characters/{id}/asi-choice`).
2. **Deskripsi Singkat** fungsi endpoint.
3. **Persyaratan Auth** (misal: `Auth: Bearer Token` / `None`).
4. **Contoh Request Body** (dalam format JSON valid lengkap dengan penjelasan tipe field, jika endpoint membutuhkan body).
5. **Contoh Response Body Sukses** (dalam format JSON valid beserta status code, contoh `200 OK` atau `201 Created`).
6. **Daftar Kemungkinan Error Response** (status code + deskripsi penyebab error, misal `400 Bad Request`, `401 Unauthorized`, `404 Not Found`).
