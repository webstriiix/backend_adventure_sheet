-- Create character_asi_choices table for ASI/feat choices at specific levels
CREATE TABLE character_asi_choices (
    id              BIGSERIAL PRIMARY KEY,
    character_id    UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    level           INT  NOT NULL,  -- character level when choice was made
    bump_str        INT  NOT NULL DEFAULT 0,
    bump_dex        INT  NOT NULL DEFAULT 0,
    bump_con        INT  NOT NULL DEFAULT 0,
    bump_int        INT  NOT NULL DEFAULT 0,
    bump_wis        INT  NOT NULL DEFAULT 0,
    bump_cha        INT  NOT NULL DEFAULT 0,
    feat_id         INT  REFERENCES feats(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    UNIQUE (character_id, level)  -- one choice per character per level
);

CREATE INDEX idx_character_asi_choices_character_id ON character_asi_choices(character_id);