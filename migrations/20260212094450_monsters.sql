-- ============================================================
-- MONSTERS
-- ============================================================
CREATE TABLE monsters (
    id          SERIAL PRIMARY KEY,
    name        TEXT NOT NULL,
    source_id   INT NOT NULL REFERENCES sources(id),
    size        TEXT[],
    type        TEXT,           -- "monstrosity","beast","undead"
    alignment   TEXT[],
    ac          JSONB,          -- [{ac:16, from:["natural armor"]}]
    hp_average  INT,
    hp_formula  TEXT,           -- "9d8 + 27"
    speed       JSONB,          -- {walk:30, climb:30}
    str INT, dex INT, con INT, int INT, wis INT, cha INT,
    skills      JSONB,
    senses      TEXT[],
    passive     INT,
    cr          TEXT,           -- "2", "1/2", "17"
    traits      JSONB,
    actions     JSONB,
    reactions   JSONB,
    UNIQUE(name, source_id)
);
