-- ============================================================
-- SUBCLASS FEATURES  (Arcane Ward, Projected Ward, etc.)
-- ============================================================
CREATE TABLE subclass_features (
    id           SERIAL PRIMARY KEY,
    name         TEXT  NOT NULL,
    source_id    INT   NOT NULL REFERENCES sources(id),
    subclass_id  INT   NOT NULL REFERENCES subclasses(id),
    level        INT   NOT NULL,
    header       INT,
    entries      JSONB NOT NULL DEFAULT '[]',
    UNIQUE(name, source_id, subclass_id)
);

CREATE INDEX idx_subclass_features_subclass_id ON subclass_features(subclass_id);
