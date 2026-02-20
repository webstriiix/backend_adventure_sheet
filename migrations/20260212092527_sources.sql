-- ============================================================
-- SOURCES  (PHB, ObojimaTallGrass, EGW, TCE, etc.)
-- ============================================================
CREATE TABLE sources (
    id          SERIAL PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,   -- "PHB", "ObojimaTallGrass"
    full_name   TEXT NOT NULL,
    is_homebrew BOOLEAN NOT NULL DEFAULT FALSE,
    publisher   TEXT,                   -- "Wizards", "1985 Games"
    page_url    TEXT
);
