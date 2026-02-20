-- ============================================================
-- SUBCLASSES  (School of Abjuration, Circle of the Petal, etc.)
-- ============================================================
CREATE TABLE subclasses (
    id              SERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    short_name      TEXT NOT NULL,
    source_id       INT  NOT NULL REFERENCES sources(id),
    class_id        INT  NOT NULL REFERENCES classes(id),
    unlock_level    INT  NOT NULL DEFAULT 3,
    fluff_text      TEXT,
    fluff_image_url TEXT,
    UNIQUE(short_name, source_id, class_id)
);

CREATE INDEX idx_subclasses_class_id ON subclasses(class_id);
