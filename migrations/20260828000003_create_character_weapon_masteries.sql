-- Create character_weapon_masteries table for weapon mastery choices
CREATE TABLE character_weapon_masteries (
    id              BIGSERIAL PRIMARY KEY,
    character_id    UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    mastery_name    TEXT NOT NULL,
    source_level    INT  NOT NULL,  -- level at which mastery was gained
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_character_weapon_masteries_character_id ON character_weapon_masteries(character_id);