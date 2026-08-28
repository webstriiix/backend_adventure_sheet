# Error Logging Template

Tiap error tak terduga yang ditemukan di lapangan wajib didokumentasikan di sini untuk keperluan autopsi dan pencegahan.

---

## [YYYY-MM-DD] Judul Singkat Error

**Gejala:** (Deskripsi apa yang terjadi di frontend/backend saat error terjadi)
**Root Cause:** (Mengapa hal ini bisa terjadi di level kode/DB?)
**Fix:** (Solusi yang diimplementasikan)
**Prevention Rule:** (Aturan baru untuk mencegah hal ini terulang)

---

## Contoh Dummy:

## [2026-05-20] Race Condition pada ASI Choice
**Gejala:** User dapat memilih 2 feat yang sama pada level yang sama karena request dikirim dua kali secara bersamaan oleh client.
**Root Cause:** Transaksi database tidak mengunci kunci `character_asi_choices` unik per level.
**Fix:** Menambah unique constraint pada kolom `(character_id, level)` di tabel `character_asi_choices`.
**Prevention Rule:** Gunakan database transaction logic dengan `FOR UPDATE` saat memvalidasi slot yang sifatnya unik.
