-- Track Death Saves and Currency in existing characters table
ALTER TABLE characters 
ADD COLUMN death_saves_successes INT NOT NULL DEFAULT 0,
ADD COLUMN death_saves_failures INT NOT NULL DEFAULT 0,
ADD COLUMN cp INT NOT NULL DEFAULT 0,
ADD COLUMN sp INT NOT NULL DEFAULT 0,
ADD COLUMN ep INT NOT NULL DEFAULT 0,
ADD COLUMN gp INT NOT NULL DEFAULT 0,
ADD COLUMN pp INT NOT NULL DEFAULT 0;

-- Track expended spell slots
CREATE TABLE character_spell_slots (
    character_id UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    slot_level INT NOT NULL, -- 1-9
    expended INT NOT NULL DEFAULT 0,
    PRIMARY KEY (character_id, slot_level)
);

-- Track expended Hit Dice
CREATE TABLE character_hit_dice (
    character_id UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    die_size INT NOT NULL, -- 6, 8, 10, or 12
    expended INT NOT NULL DEFAULT 0,
    PRIMARY KEY (character_id, die_size)
);
