# Error Log & Incidents

## [2026-08-31] Non-Interactive Script Hang Due to `docker exec -t` and Missing Subprocess Timeout

- **Gejala**: Script Python (`tests/smoke_test.py`) yang me-run `docker exec -t` untuk query `psql` non-interaktif hang/macet lebih dari 30 menit tanpa pernah selesai atau memberikan error output.
- **Root Cause**: Flag `-t` pada command `docker exec` mengalokasikan pseudo-TTY. Dikombinasikan dengan binary `psql` yang mendeteksi TTY dan mengaktifkan pager (serta mode interaktif lainnya), psql menunggu input stdin dari TTY. Karena script berjalan dari `subprocess.run` tanpa stdin TTY interaktif, proses menggantung selamanya menunggu input yang tidak pernah datang. Selain itu, `subprocess.run` tidak memiliki parameter `timeout=` eksplisit.
- **Fix**: 
  1. Mengganti `docker exec -t` dengan `docker compose exec -T` (huruf `-T` besar = eksplisit menonaktifkan pseudo-TTY) atau `docker exec` tanpa flag `-t`.
  2. Menambahkan `-P pager=off` pada argumen `psql` untuk memastikan pager nonaktif secara eksplisit.
  3. Menambahkan parameter `timeout=30` eksplisit pada setiap pemanggilan `subprocess.run(...)`.
- **Prevention Rule**: SEMUA script otomatis (Python/Bash) yang memanggil `docker exec` atau `docker compose exec` untuk query/command non-interaktif (misal: smoke test, CI/CD) WAJIB menggunakan `-T` (bukan `-t`), WAJIB menonaktifkan pager psql (`-P pager=off`), dan WAJIB menetapkan `timeout` eksplisit pada setiap pemanggilan `subprocess.run`.

## [2026-08-31] Class/Subclass Features Overwritten Due to Missing `level` in Unique Constraint

- **Gejala**: Data `class_features` (misal Wizard XPHB) kehilangan baris untuk fitur yang memiliki nama sama di level berbeda (misal `Ability Score Improvement` yang hanya menyisakan 1 baris di level 16 alih-alih 4 baris di level 4, 8, 12, 16; dan `Subclass Feature` yang hanya menyisakan 1 baris di level 14 alih-alih 3 baris di level 6, 10, 14).
- **Root Cause**: Unique constraint lama (`class_features_name_source_id_class_id_key` pada `(class_id, source_id, name)`) tidak menyertakan kolom `level`. Saat importer menjalankan `INSERT ... ON CONFLICT (class_id, source_id, name) DO UPDATE`, fitur dengan nama sama di level berikutnya menimpa (overwrite) baris level sebelumnya.
- **Fix**: 
  1. Membuat migration baru `20260831000002_fix_class_subclass_features_unique_constraint.sql` untuk drop constraint lama dan menambahkan unique constraint baru `UNIQUE (class_id, source_id, name, level)` pada `class_features` dan `UNIQUE (subclass_id, source_id, name, level)` pada `subclass_features`.
  2. Meng-update ON CONFLICT clause pada `src/importers/import_classes.rs` (baris 79 dan 199) menjadi `ON CONFLICT (class_id, source_id, name, level)`.
  3. Melakukan `TRUNCATE class_features, subclass_features` dan re-import data via `./import.sh`.
- **Prevention Rule**: SETIAP tabel yang menyimpan data per-level (`class_features`, `subclass_features`, dan tabel serupa di masa depan) WAJIB punya UNIQUE constraint yang menyertakan `level` sebagai bagian kombinasi unique key, BUKAN cuma `(entity_id, source_id, name)` — karena D&D 5e banyak memakai nama fitur yang sengaja berulang di level berbeda (`Ability Score Improvement`, `Subclass Feature`, dll). Smoke test WAJIB menyertakan assertion row-count eksplisit untuk fitur bernama sama, bukan cuma cek keberadaan nama di satu level tertentu.
