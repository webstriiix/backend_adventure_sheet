-- ============================================================
-- SUBRACES  (High Elf, Hill Dwarf, etc.)
-- ============================================================
CREATE TABLE subraces (
    id              SERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    source_id       INT NOT NULL REFERENCES sources(id),
    race_id         INT NOT NULL REFERENCES races(id),
    speed           JSONB,                    -- NULL = inherit from race
    ability_bonuses JSONB,
    entries         JSONB NOT NULL DEFAULT '[]',
    UNIQUE(name, source_id, race_id)
);
