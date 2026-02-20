-- ============================================================
-- REWARDS  (Blessings, Charms, special homebrew rewards)
-- ============================================================
CREATE TABLE rewards (
    id          SERIAL PRIMARY KEY,
    name        TEXT NOT NULL,
    source_id   INT NOT NULL REFERENCES sources(id),
    type        TEXT,           -- "Blessing","Charm","Gift"
    entries     JSONB NOT NULL,
    UNIQUE(name, source_id)
);
