-- ============================================================
-- OPTIONAL FEATURES  (Fighting Styles, Metamagic, Invocations)
-- ============================================================
CREATE TABLE optional_features (
    id              SERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    source_id       INT NOT NULL REFERENCES sources(id),
    feature_type    TEXT NOT NULL,  -- "FS:F","MM","EI","BOON" etc.
    prerequisite    JSONB,
    entries         JSONB NOT NULL,
    UNIQUE(name, source_id, feature_type)
);
