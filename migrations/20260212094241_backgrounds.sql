-- ============================================================
-- BACKGROUNDS  (Apprentice Diver, Acolyte, etc.)
-- ============================================================
CREATE TABLE backgrounds (
    id                  SERIAL PRIMARY KEY,
    name                TEXT NOT NULL,
    source_id           INT NOT NULL REFERENCES sources(id),
    skill_proficiencies JSONB,
    tool_proficiencies  JSONB,
    language_count      INT DEFAULT 0,
    starting_equipment  JSONB,
    entries             JSONB NOT NULL,
    UNIQUE(name, source_id)
);
