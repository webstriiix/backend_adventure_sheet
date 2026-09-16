# Agent Rules & Guidelines

## Tooling & Scripting Guidelines

1. **Non-Interactive Execution in Automation Scripts**:
   SETIAP script otomatis (Python/Bash) yang memanggil `docker exec` atau `docker compose exec` untuk query non-interaktif (misal smoke test, CI) WAJIB pakai `-T` (bukan `-t`), WAJIB nonaktifkan pager psql (`-P pager=off`), dan WAJIB set timeout eksplisit pada setiap subprocess call. Rujuk `agent/errors.md` untuk insiden yang melatarbelakangi aturan ini.

2. **Per-Level Data Unique Constraints & Smoke Test Assertions**:
   SETIAP tabel yang menyimpan data per-level (`class_features`, `subclass_features`, dan tabel serupa) WAJIB menyertakan kolom `level` dalam UNIQUE constraint (misal: `UNIQUE (class_id, source_id, name, level)`), bukan hanya `(entity_id, source_id, name)`. Hal ini untuk mencegah penimpaan data (overwrite) pada fitur D&D yang memiliki nama sama di berbagai level (`Ability Score Improvement`, `Subclass Feature`). Smoke test WAJIB menguji jumlah baris (row count) eksplisit untuk fitur berulang tersebut.
