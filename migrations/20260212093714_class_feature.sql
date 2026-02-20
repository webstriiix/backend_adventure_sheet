-- ============================================================
-- CLASS FEATURES  (Arcane Recovery, Action Surge, etc.)
-- ============================================================
CREATE TABLE class_features (
    id               SERIAL PRIMARY KEY,
    name             TEXT    NOT NULL,
    source_id        INT     NOT NULL REFERENCES sources(id),
    class_id         INT     NOT NULL REFERENCES classes(id),
    level            INT     NOT NULL,
    entries          JSONB   NOT NULL DEFAULT '[]',
    is_subclass_gate BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(name, source_id, class_id)
);

CREATE INDEX idx_class_features_class_id ON class_features(class_id);
