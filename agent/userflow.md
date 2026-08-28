# Request Userflow & Data Processing Pipelines — Backend

Dokumen ini menjelaskan alur data teknis (*request execution flow*) dari sudut pandang backend saat melayanan permintaan dari frontend client.

> **Catatan Penting Multi-Client:** Istilah "Client" dalam seluruh alur di bawah ini merujuk secara fleksibel ke salah satu dari **4 jenis konsumen API** (CLI via Ratatui, atau Desktop, Web, dan Mobile via Dioxus). Backend **tidak membedakan perlakuan atau mengembalikan format berbeda** berdasarkan tipe client yang melakukan HTTP request. Seluruh alur request, validasi, dan response JSON bersifat identik dan platform-agnostic.

---

## 1. Alur Read State Karakter & Progression Manifest

Alur ketika client meminta data karakter lengkap (`GET /api/v1/characters/{id}` atau `GET /api/v1/characters/{id}/progression`).

```
[ Client (CLI / Desktop / Web / Mobile) ]
       │
       │ 1. GET /api/v1/characters/{id} (Header: Authorization Bearer <token>)
       ▼
[ Axum Auth Extractor ]
       │
       │ 2. Verifikasi JWT Token -> Extract user_id
       ▼
[ Handler: characters::core::get_character ]
       │
       │ 3. Fetch Raw Data dari Postgres (sqlx pool)
       │    - Character Row (Ability Scores, Current HP, XP, Base Info)
       │    - Character Classes & Subclasses
       │    - Feats, Background, Race & Race Options
       │    - Active Proficiencies, Inventory Items, Spells
       ▼
[ Service: Rules Engine & Stat Calculator ]
       │
       │ 4. Kalkulasi Derived Stats (In-Memory Rust Logic):
       │    - Ability Modifiers: floor((score - 10) / 2)
       │    - Total Level & Proficiency Bonus (+2 pada lv 1-4, +3 pada lv 5-8, dst)
       │    - Max HP (Base Class Hit Die + CON mod per level + Feat bonuses like Tough)
       │    - Armor Class (10 + DEX mod + Armor/Shield + Fighting Style bonuses)
       │    - Saving Throws (Ability Mod + Prof Bonus jika proficient)
       │    - Skill Bonuses (Ability Mod + Prof/Expertise Bonus)
       │    - Attack Hit Bonuses & Damage Formulas per equipped weapon
       ▼
[ Service: Progression Manifest Engine ]
       │
       │ 5. Evaluasi Progression Manifest (Level 1–20):
       │    - Cocokkan level karakter dengan slot ASI/Feat, Subclass unlock, Weapon Mastery.
       │    - Tandai status per slot: `completed`, `partial`, `pending`, atau `locked`.
       ▼
[ JSON Response ]
       │
       │ 6. Return 200 OK dengan Payload Terkalkulasi Final
       ▼
[ Client (CLI / Desktop / Web / Mobile) ] -> Render UI masing-masing platform.
```

---

## 2. Alur Submit Keputusan / Decision Flow

Alur saat pengguna menentukan pilihan (misal: memilih ASI/Feat pada Level 4 atau memilih Subclass pada Level 3).

```
[ Client (CLI / Desktop / Web / Mobile) ]
       │
       │ 1. POST /api/v1/characters/{id}/asi-choice (Payload: bump_str: 2, level: 4, dll)
       ▼
[ Axum Auth Extractor ]
       │
       │ 2. Verifikasi JWT -> Pastikan user_id sesuai pemilik karakter
       ▼
[ Handler: characters::asi::post_asi_choice ]
       │
       │ 3. Buka Database Transaction (tx)
       ▼
[ Service: Validation Engine ]
       │
       │ 4. Validasi Aturan D&D 5e:
       │    a. Apakah total level karakter >= level slot keputusan (misal: Level 4)?
       │    b. Apakah level tersebut valid untuk ASI/Feat kelas karakter tersebut (baca column `asi_levels`)?
       │    c. Apakah slot level ini sudah pernah diisi sebelumnya? (Mencegah double-spending)
       │    d. Jika memilih Feat: Apakah karakter memenuhi prerequisite Feat (min stat, spellcasting, race requirement)?
       │    e. Jika memilih ASI: Apakah total stat bump <= 2 dan stat max <= 20?
       │
       ├─── [ Validasi Gagal ] ──> Rollback tx -> Return 400 Bad Request (JSON error detail)
       │
       ▼ [ Validasi Sukses ]
[ DB Query Layer ]
       │
       │ 5. Execute INSERT INTO character_asi_choices / character_feats dalam transaction.
       │ 6. Commit Transaction.
       ▼
[ Service: Manifest Re-evaluator ]
       │
       │ 7. Recalculate Progression Manifest & Derived Stats terbaru.
       ▼
[ JSON Response ]
       │
       │ 8. Return 200 OK dengan Manifest & Stat Karakter ter-update.
       ▼
[ Client (CLI / Desktop / Web / Mobile) ] -> Update UI state platform secara instan.
```

---

## 3. Alur Rest & Resource Management Flow

Alur ketika karakter melakukan Istirahat (Short Rest / Long Rest):

### Short Rest (`POST /api/v1/characters/{id}/short-rest`)
1. Client mengirim payload jumlah hit dice yang di-spend per ukuran dadu (misal `{ "8": 1 }`).
2. Backend memvalidasi ketersediaan hit dice yang belum terpakai (`expended < total_hit_dice`).
3. Backend menghitung penyembuhan HP: `dadu_roll + CON_modifier`. Update `current_hp` (tidak melebihi `max_hp`).
4. Reset fitur yang memiliki mekanisme `recharge_on = 'short_rest'` (misal: Action Surge, Channel Divinity).
5. Simpan transaksi dan return status karakter terbaru.

### Long Rest (`POST /api/v1/characters/{id}/long-rest`)
1. Reset `current_hp` menjadi sama dengan `max_hp`.
2. Reset seluruh `spell_slots` terpakai menjadi 0.
3. Reset seluruh hit dice terpakai (sesuai aturan 2024 recovery hit dice).
4. Reset seluruh penggunaan fitur yang recharge pada `short_rest` maupun `long_rest`.
5. Return status karakter terbaru.

---

## 4. Alur Error & Edge Case Handling

### A. Race Condition / Concurrency Control
- **Skenario:** Dua request bersamaan untuk submit keputusan pada slot level yang sama.
- **Backend Protection:** Menggunakan Database Unique Constraint `(character_id, level)` pada tabel `character_asi_choices` dan transaksi database berisolasi. Jika terjadi pelanggaran unique constraint, Postgres mengembalikan error duplicate yang ditangkap backend dan diubah menjadi `400 Bad Request: Choice for this level already exists`.

### B. Auth / Unauthorized Access
- Request tanpa header `Authorization` -> `401 Unauthorized`.
- Request mencoba mengakses/mengedit `character_id` milik user lain -> `404 Not Found` (mencegah penyingkapan keberadaan ID karakter milik user lain / ID enumeration attack).

### C. Validation Error Payload Standard
Setiap kesalahan validasi aturan mengembalikan format JSON konsisten:
```json
{
  "error": "Pesan kesalahan yang mudah dipahami oleh developer/client"
}
```
Raw database error (seperti `pg_error: FK constraint failed ...`) **TIDAK BOLEH** langsung dikirim ke client.
