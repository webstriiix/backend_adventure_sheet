# User Flow & Arsitektur Interaksi

"Client" dalam seluruh alur di bawah ini dapat berupa aplikasi **CLI, Desktop, Web, atau Mobile**. Backend tidak membedakan perlakuan antar platform.

## Alur Data API (Client -> Backend)
1. **Request**: Client mengirimkan HTTP request ke endpoint (misalnya query Progression karakter, daftar Spell, atau sisa Hit Dice).
2. **Validasi Auth**: Backend memverifikasi authorization token via middleware JWT.
3. **Query & Kalkulasi**: Backend menarik data mentah dari PostgreSQL, kemudian mengeksekusi *business logic* dan kalkulasi D&D 2024 (seperti modifier, total maksimum kalkulasi, stack buffs).
4. **Respon (Final Manifest)**: Backend mengembalikan data JSON ke client dalam bentuk yang **SUDAH final terhitung**. Frontend dilarang keras melakukan kalkulasi mendalam terkait aturan D&D 5e — ia hanya bertugas merender data yang diberikan backend.

## Alur Keputusan (Decision Slot)
Saat pemain naik level atau membuat karakter, terdapat slot keputusan (memilih skill, subclass, wizard school, feat, ability boost).
1. **Submit**: Client mengirimkan payload pilihan ke backend via endpoint Progression/Race/Class choices.
2. **Validasi Rule**: Backend memeriksa validitas tindakan secara aturan ketat:
   - Apakah *character level* sudah memadai berdasarkan Progression Manifest?
   - Apakah prasyarat Feat/Trait terpenuhi?
   - Apakah item ini valid untuk kelas/ras ini?
   - Apakah pilihan ini belum pernah dipilih?
3. **Simpan State**: Backend mencatat riwayat ke tabel riwayat (misalnya `character_asi_choices` atau `character_proficiencies`).
4. **Return Manifest**: Backend membalas dengan Progression Manifest karakter yang telah diperbarui secara penuh, memicu Frontend merender status baru.

## Alur Workflow Import Data
Untuk mem-bootstrap atau me-reset ruleset database backend:
1. Jalankan instance backend (biasanya di port `:8080`).
2. Jalankan script root `./import.sh` di lokal.
3. Script akan menyuntikkan file JSON 5etools ke endpoint `POST /api/v1/import`.
4. **Importer `src/importers/`** mem-parsing JSON, memastikan ia memfilter secara ketat *source* + *edition* yang benar.
5. Importer memasukkan/merubah data ke Postgres dengan argumen PostgreSQL `ON CONFLICT DO UPDATE`. 
   *Penting*: `ON CONFLICT` **WAJIB** menyertakan semua kolom pembeda relevan, termasuk indeks **`level`** sebagai constraint kunci (lihat `agent/rules.md` untuk deskripsi historis error terkait ini).
6. Database diisi dengan Compendium ruleset, siap dikonsumsi oleh endpoints.
