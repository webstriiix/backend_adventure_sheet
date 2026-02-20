-- ============================================================
-- USER'S CHARACTER'S SHEET 
-- ============================================================

CREATE TABLE characters (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    experience_pts  INT  NOT NULL DEFAULT 0,
    race_id         INT  REFERENCES races(id),
    subrace_id      INT  REFERENCES subraces(id),
    background_id   INT  REFERENCES backgrounds(id),
    str             INT  NOT NULL DEFAULT 8,
    dex             INT  NOT NULL DEFAULT 8,
    con             INT  NOT NULL DEFAULT 8,
    int             INT  NOT NULL DEFAULT 8,
    wis             INT  NOT NULL DEFAULT 8,
    cha             INT  NOT NULL DEFAULT 8,
    max_hp          INT  NOT NULL DEFAULT 0,
    current_hp      INT  NOT NULL DEFAULT 0,
    temp_hp         INT  NOT NULL DEFAULT 0,
    inspiration     BOOLEAN NOT NULL DEFAULT FALSE,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Chosen feats per character
CREATE TABLE character_feats (
    id              SERIAL PRIMARY KEY,
    character_id    UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    feat_id         INT  NOT NULL REFERENCES feats(id),

    -- Which ASI did the player pick? (e.g. Coven Witch lets you pick int OR wis)
    chosen_ability  TEXT,           -- "int", "wis", "con" etc. NULL if feat has no ASI

    -- For feats that grant spells with limited uses (Bumbling Fool: 3/long rest)
    uses_remaining  INT,            -- NULL if feat has no tracked uses
    uses_max        INT,            -- NULL if feat has no tracked uses
    recharge_on     TEXT,           -- "long_rest" | "short_rest" | "dawn" | NULL

    -- Which source gave them this feat (ASI level-up, background, species, etc.)
    source_type     TEXT NOT NULL DEFAULT 'level',  -- "level" | "background" | "species" | "bonus"
    gained_at_level INT,            -- which character level they took it

    UNIQUE (character_id, feat_id)  -- one copy per character (unless you want stacking, rare)
);
-- Chosen spells per character
CREATE TABLE character_spells (
    character_id UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    spell_id     INT NOT NULL REFERENCES spells(id),
    is_prepared  BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (character_id, spell_id)
);

-- Inventory
CREATE TABLE character_inventory (
    id           SERIAL PRIMARY KEY,
    character_id UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    item_id      INT NOT NULL REFERENCES items(id),
    quantity     INT DEFAULT 1,
    is_equipped  BOOLEAN DEFAULT FALSE,
    is_attuned   BOOLEAN DEFAULT FALSE,
    notes        TEXT
);

-- Support MultiClass
CREATE TABLE character_classes (
    character_id UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    class_id     INT  NOT NULL REFERENCES classes(id),
    subclass_id  INT  REFERENCES subclasses(id),  -- NULL until unlock level
    level        INT  NOT NULL DEFAULT 1,
    is_primary   BOOLEAN NOT NULL DEFAULT FALSE,   -- determines hit die for HP at level 1
    PRIMARY KEY (character_id, class_id)
);

CREATE INDEX idx_characters_user_id             ON characters(user_id);
CREATE INDEX idx_character_classes_character_id ON character_classes(character_id);
CREATE INDEX idx_character_feats_character_id   ON character_feats(character_id);
CREATE INDEX idx_character_spells_character_id  ON character_spells(character_id);
CREATE INDEX idx_character_inventory_char_id    ON character_inventory(character_id);
