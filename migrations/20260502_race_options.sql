-- Race options (choices available to player for a race or subrace)
CREATE TABLE race_options (
    id SERIAL PRIMARY KEY,
    race_id INT REFERENCES races(id),
    subrace_id INT REFERENCES subraces(id),
    source_id INT NOT NULL REFERENCES sources(id),
    option_type TEXT NOT NULL, -- 'feat'|'cantrip'|'spell'|'skill'|'darkvision'|'variable_trait'
    choices JSONB,            -- array of choice objects, or NULL meaning free-form choice
    min_choose INT NOT NULL DEFAULT 1,
    max_choose INT NOT NULL DEFAULT 1,
    note TEXT,
    UNIQUE(race_id, subrace_id, source_id, option_type)
);

CREATE INDEX idx_race_options_race_subrace ON race_options(race_id, subrace_id);

-- Persist character selections for these options
CREATE TABLE character_race_options (
    id SERIAL PRIMARY KEY,
    character_id UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    race_option_id INT NOT NULL REFERENCES race_options(id) ON DELETE CASCADE,
    selection JSONB NOT NULL,
    UNIQUE(character_id, race_option_id)
);
