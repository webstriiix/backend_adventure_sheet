-- ============================================================
-- RACES  (Dara, Elf, Dwarf, etc.)
-- ============================================================
CREATE TABLE races (
    id                      SERIAL PRIMARY KEY,
    name                    TEXT NOT NULL,
    source_id               INT NOT NULL REFERENCES sources(id),
    size                    TEXT[] NOT NULL DEFAULT '{}',    -- ["S","M"]
    speed                   JSONB  NOT NULL DEFAULT '{"walk": 30}',
    ability_bonuses         JSONB NOT NULL DEFAULT '{}',     -- weighted choose or fixed
    age_description         TEXT,
    alignment_description   TEXT,
    skill_proficiencies     JSONB,
    language_proficiencies  JSONB,
    trait_tags              TEXT[] NOT NULL DEFAULT '{}',
    entries                 JSONB NOT NULL DEFAULT '[]',      -- full rich text
    UNIQUE(name, source_id)
);
