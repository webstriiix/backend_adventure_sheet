-- ============================================================
-- SPELLS
-- ============================================================
CREATE TABLE spells (
    id                  SERIAL PRIMARY KEY,
    name                TEXT NOT NULL,
    source_id           INT NOT NULL REFERENCES sources(id),
    level               INT NOT NULL,           -- 0 = cantrip
    school              TEXT NOT NULL,          -- "C","V","A","N","I","E","T","D"
    casting_time        JSONB NOT NULL DEFAULT '[]', -- [{number:1, unit:"action"}]
    range               JSONB NOT NULL DEFAULT '{}',
    components          JSONB NOT NULL DEFAULT '{}', -- {v:true, s:true, m:"powder"}
    duration            JSONB NOT NULL DEFAULT '[]',
    entries             JSONB NOT NULL DEFAULT '[]',
    entries_higher_lvl  JSONB,
    ritual              BOOLEAN NOT NULL DEFAULT FALSE,
    concentration       BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(name, source_id)
);

-- Spell ↔ Class join (spells available to which classes)
CREATE TABLE spell_classes (
    spell_id    INT NOT NULL REFERENCES spells(id),
    class_id    INT NOT NULL REFERENCES classes(id),
    PRIMARY KEY (spell_id, class_id)
);

CREATE INDEX idx_spells_level  ON spells(level);
CREATE INDEX idx_spells_school ON spells(school);
