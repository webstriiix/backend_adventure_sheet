-- ============================================================
-- FEATS
-- ============================================================
CREATE TABLE feats (
    id                  SERIAL PRIMARY KEY,
    name                TEXT NOT NULL,
    source_id           INT  NOT NULL REFERENCES sources(id),
    page                INT,
    prerequisite        JSONB,
    ability             JSONB,          -- ASI granted by this feat
    skill_proficiencies JSONB,
    resist              TEXT[],
    additional_spells   JSONB,
    has_uses            BOOLEAN NOT NULL DEFAULT FALSE,
    uses_formula        TEXT,           -- "3" | "@prof" | NULL
    recharge_on         TEXT,           -- "long_rest" | "short_rest" | NULL
    entries             JSONB NOT NULL,
    UNIQUE(name, source_id)
);

