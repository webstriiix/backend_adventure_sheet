-- Character proficiencies and expertise tracking
CREATE TABLE character_proficiencies (
    id SERIAL PRIMARY KEY,
    character_id UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    category TEXT NOT NULL,             -- 'saving_throw' or 'skill'
    name TEXT NOT NULL,                 -- e.g., 'wisdom', 'athletics', 'perception'
    proficiency_type TEXT NOT NULL DEFAULT 'proficiency', -- 'proficiency' or 'expertise'
    UNIQUE(character_id, category, name)
);

CREATE INDEX idx_character_proficiencies_character ON character_proficiencies(character_id);
