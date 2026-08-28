# Product Requirement Document (PRD) — CLI Adventure Sheet Backend

## 1. Visi & Tujuan Utama Backend

Backend **CLI Adventure Sheet** adalah "otak" terpusat dan *authoritative single source of truth* untuk aplikasi manajemen karakter D&D 5e (khususnya edisi 2024 / XPHB). Backend bertanggung jawab penuh atas seluruh kalkulasi aturan D&D 5e, penyimpanan state karakter, serta penegakan validasi aturan game (rules engine).

### Konsumen API & Multi-Platform Support:
API ini dikonsumsi oleh 4 jenis frontend client (CLI, Desktop, Web, Mobile), semuanya harus diperlakukan setara — tidak ada endpoint atau response khusus per platform.

### Prinsip Utama Backend:
1. **Zero Math on Frontend:** Client frontend (Ratatui CLI atau Dioxus Desktop/Web/Mobile) **hanya bertugas merender data**. Backend tidak pernah mengembalikan angka mentah (*raw values*) yang butuh kalkulasi ulang di frontend. Seluruh modifier ability score, total bonus serangan, saving throw, total HP, spell slot tersisa, dan kalkulasi proficiency bonus harus dihitung final oleh backend.
2. **Platform-Agnostic & Equal Treatment:** API dirancang fully platform-agnostic. Tidak ada asumsi lingkungan run-time client (misalnya browser vs terminal). Autentikasi sepenuhnya bergantung pada HTTP Bearer JWT Header (`Authorization: Bearer <token>`) tanpa bergantung pada Cookie atau browser storage.
3. **Identical Progression Manifest Structure:** Respon Progression Manifest yang di-return identik untuk keempat target client. Perbedaan visual sepenuhnya menjadi tanggung jawab masing-masing client frontend.
4. **Authoritative Rule Enforcement:** Backend menolak semua aksi illegal (misal: mengambil feat yang tidak memenuhi prerequisit, memilih 2 feat yang sama jika tidak diizinkan, mengambil ASI di luar jadwal level kelas, atau menaikkan level melebih total 20).
5. **Progression Manifest Centric:** Backend mengelola seluruh riwayat pertumbuhan karakter dari level 1 hingga 20 menggunakan struktur **Progression Manifest**.

---

## 2. Pengertian & Spesifikasi Progression Manifest

**Progression Manifest** adalah data struktur hierarkis deterministik yang mencatat seluruh titik keputusan (*decision points*) dan riwayat pilihan karakter sepanjang perjalanan level 1–20. 

### Komponen Utama Progression Manifest:
- **Level Range (1–20):** Mewakili urutan perkembangan karakter.
- **Decision Points (Slot Keputusan):** Setiap level memiliki kumpulan slot pilihan wajib atau opsional berdasarkan Kombinasi Ras, Background, Kelas Utama, Multiclass, dan Subclass.
- **Jenis Keputusan (*Choice Types*):**
  - `asi`: Ability Score Improvement (+2 pada satu stat, atau +1 pada dua stat).
  - `feat`: Pemilihan Origin Feat / General Feat (opsional menggantikan ASI pada level tertentu).
  - `subclass`: Pemilihan Subclass saat mencapai unlock level kelas (contoh: Level 3 untuk XPHB).
  - `skill_proficiency` & `expertise`: Pilihan kemahiran skill dari daftar kelas/background.
  - `weapon_mastery`: Pilihan penguasaan senjata (Cleave, Push, Graze, dll) sesuai tabel kelas Fighter/Barbarian/Rogue/dll.
  - `spell_selection`: Pemilihan cantrip, spell known, atau spell prepared per level.
  - `race_option`: Pilihan varian ras/spesies (cantrip opsional, resistensi, dll).

### Status Slot Keputusan (*Slot Statuses*):
Setiap `decision_point` dalam manifest memiliki salah satu status berikut:
1. **`completed`**: Pilihan untuk level ini sudah terisi lengkap sesuai jumlah minimum `required_count`.
2. **`partial`**: Pilihan sudah diisi sebagian tetapi belum memenuhi `required_count` (misal: baru memilih 2 dari 3 Weapon Mastery).
3. **`pending`**: Karakter sudah mencapai level ini, namun keputusan belum diisi sama sekali.
4. **`locked`**: Karakter belum mencapai level tersebut (contoh: slot level 4 pada karakter level 2).

---

## 3. Arsitektur API Utama (High-Level)

Detail teknis handler dan skema payload diatur di `tech.md` dan `structure.md`. Berikut adalah kelompok endpoint utama backend:

### A. Authentication & User Management
- `POST /api/v1/signup`: Registrasi akun baru (password dikompresi dengan Argon2id).
- `POST /api/v1/login`: Otentikasi user dan penerbitan JWT Bearer Token (berlaku 30 hari).

### B. Compendium (Public Data Read-Only)
Dataset D&D 5e XPHB publik untuk dikonsumsi frontend tanpa otentikasi:
- `GET /api/v1/classes` & `GET /api/v1/classes/{name}/{source}`: Detail kelas, tabel progres spell slot, subclass gate, feature list.
- `GET /api/v1/races` & `GET /api/v1/races/{name}/{source}/options`: Spesies, sub-ras, dan opsi varian ras.
- `GET /api/v1/backgrounds`: Latar belakang karakter beserta bonus stat & bonus feat default.
- `GET /api/v1/spells`: Database spell dengan filter nama, level, dan source.
- `GET /api/v1/items`: Database item, senjata, armor, dan perlengkapan.
- `GET /api/v1/monsters`: Bestiary data.
- `GET /api/v1/optional-features`: Eldritch Invocations, Fighting Styles, Metamagic, dll.

### C. Character Core & State Management (Auth Required)
- `GET /api/v1/characters`: Daftar karakter milik user terautentikasi.
- `POST /api/v1/characters`: Membuat karakter dasar level 1 (nama, stat awal, ras, kelas, background).
- `GET /api/v1/characters/{id}`: Mengambil data stat terhitung final (derived stats: AC, Max HP, Current HP, Modifiers, Passive Perception, Saves, Skill Bonuses).
- `PUT /api/v1/characters/{id}`: Update state umum karakter.
- `DELETE /api/v1/characters/{id}`: Hapus karakter cascade.

### D. Progression & Decision Management (Auth Required)
- `GET /api/v1/characters/{id}/progression`: Fetch **Progression Manifest** lengkap karakter (level 1–20) beserta status slot (`pending`, `partial`, `completed`, `locked`).
- `POST /api/v1/characters/{id}/asi-choice`: Submit keputusan ASI atau Feat pada level tertentu.
- `POST /api/v1/characters/{id}/race-options`: Submit pilihan opsi ras (cantrip/feat opsional).
- `POST /api/v1/characters/{id}/proficiencies`: Submit / update kemahiran skill & expertise.
- `PATCH /api/v1/characters/{id}/classes/{class_id}`: Naik level / atur subclass pada kelas tertentu.

### E. Combat, Spells, Inventory & Rest Tracking (Auth Required)
- `GET /api/v1/characters/{id}/actions`: Mengambil kalkulasi aksi bertarung (Attack, Bonus Action, Reaction, Spells, Limited Uses).
- `POST /api/v1/characters/{id}/spells` & `PUT /api/v1/characters/{id}/spells/{spell_id}`: Manajemen spell terdaftar & prepared status.
- `PATCH /api/v1/characters/{id}/spell-slots/{level}`: Tracking spell slot terpakai (*expended slots*).
- `POST /api/v1/characters/{id}/inventory`: Tambah / update / equip item inventory.
- `POST /api/v1/characters/{id}/short-rest`: Eksekusi Short Rest (recovery fitur SR, pemakaian hit dice).
- `POST /api/v1/characters/{id}/long-rest`: Eksekusi Long Rest (reset HP full, spell slots, hit dice, & fitur LR).

---

## 4. Batasan & Lingkup Pengembangan Fase Ini

- **Fokus Utama:** Menyediakan API backend authoritative yang stabil, platform-agnostic, dan konsisten untuk mendukung 4 target frontend sekaligus (CLI berbasis Ratatui serta Desktop, Web, dan Mobile berbasis Dioxus).
- **System Constraints:** Multi-tenant berbasis `user_id` dari JWT token. User hanya bisa membaca dan mengubah karakter miliknya sendiri. API tidak menyediakan perlakuan khusus atau endpoint percabangan berdasarkan tipe client.
