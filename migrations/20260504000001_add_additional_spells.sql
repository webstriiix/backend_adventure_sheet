ALTER TABLE classes ADD COLUMN IF NOT EXISTS additional_spells JSONB;
ALTER TABLE subclasses ADD COLUMN IF NOT EXISTS additional_spells JSONB;
