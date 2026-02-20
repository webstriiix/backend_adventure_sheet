-- ============================================================
-- ITEMS  (weapons, armor, magic items, potions, etc.)
-- ============================================================
CREATE TABLE items (
    id              SERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    source_id       INT NOT NULL REFERENCES sources(id),
    type            TEXT,       -- "W","A","P","G","$","RD" etc.
    rarity          TEXT,       -- "common","uncommon","rare","legendary"
    weight          NUMERIC,
    value_cp        INT,        -- value in copper pieces
    damage          JSONB,      -- {dmg1:"1d8", dmgType:"S"}
    armor_class     INT,
    properties      TEXT[] NOT NULL DEFAULT '{}',     -- ["Finesse","Light","Whim"]
    requires_attune BOOLEAN NOT NULL DEFAULT FALSE,
    entries         JSONB,
    is_magic        BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(name, source_id)
);
