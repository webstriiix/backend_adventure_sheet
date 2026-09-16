# Product Requirements Document (PRD)

**Repo ini adalah BACKEND dari proyek CLI Adventure Sheet (Axum, Postgres, SQLx). Untuk kode frontend (Dioxus/Ratatui, 4 target: CLI, Desktop, Web, Mobile), lihat repo terpisah.**

## Visi & Tujuan
Backend ini berfungsi sebagai "otak" tunggal (single source of truth) untuk seluruh state karakter D&D 5e (khususnya 2024/XPHB). Backend bertanggung jawab penuh atas semua kalkulasi aturan permainan (seperti modifier, HP maksimal, hitung DC, armor class) serta memelihara **Progression Manifest** — yaitu riwayat lengkap pilihan yang dibuat pemain dari level 1 hingga 20 (termasuk Ability Score Improvement, Feat, Skill Proficiency, Weapon Mastery, Spell Selection, pilihan Subclass, dll).

API backend ini dirancang agnostik terhadap client dan dikonsumsi oleh 4 jenis frontend (CLI, Desktop, Web, Mobile) yang kedudukannya sepenuhnya setara. Tidak boleh ada pembuatan endpoint spesifik untuk platform tertentu atau respons format khusus yang menguntungkan salah satu frontend di atas yang lain.

## Sumber Data
Data rules permainan menggunakan format JSON dari **5etools** (contoh: `class-wizard.json`). Data ini diimport melalui importer internal backend (`src/importers/`) masuk ke database relasional (PostgreSQL).

**Aturan Penting Importer**: 
Berdasarkan investigasi historis, data 5etools memiliki entri "transitional pointer" (menggunakan penanda `reprintedAs` atau field `_copy`) yang bisa saja basi (out-of-date) atau salah jika dibandingkan dengan entri final edisi terbaru (`edition: "one"`). Importer tidak boleh berasumsi bahwa baris pertama yang cocok namanya adalah data aktual. **Importer WAJIB mem-filter berdasarkan origin `source` dan `edition` secara eksplisit** setiap saat.

## Scope MVP vs Fitur Lanjutan

### Scope Selesai/MVP
- Sistem autentikasi pengguna.
- Parsing import parser JSON 5etools untuk Class, Subclass, Feat, Item, Spell, Race, Background.
- Pembuatan dan penyimpanan karakter dengan pemilihan Class, Race, dan Background.
- Mekanisme level up / level down dengan dukungan Progression Manifest (menyimpan riwayat pilihan seperti ASI, subclass, dan spells).
- Kalkulasi otomatis stat (HP, Ability Modifier, Save DC).
- Pelacakan sumber daya karakter (Spell slots, Hit Dice, dan resource pools).
- Endpoint RESTful lengkap yang telah diuji (tercatat di `API_DOCUMENTATION.md`).

### Fitur Lanjutan / Future Scope
- Dukungan Multi-Classing penuh (aturan tumpang tindih spellcaster, spell slots).
- Inventory & Equipment Management (termasuk attunement dan equipment modifiers).
- Sistem Homebrew custom.
- Encounter Mode / Initiative Tracker integrasi ke VTT.
