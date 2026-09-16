-- Add XPHB/2024 columns to classes table
ALTER TABLE classes
ADD COLUMN IF NOT EXISTS primary_ability TEXT[] DEFAULT '{}',
ADD COLUMN IF NOT EXISTS prepared_spells_progression INT[],
ADD COLUMN IF NOT EXISTS prepared_spells_change TEXT,
ADD COLUMN IF NOT EXISTS cantrip_progression INT[],
ADD COLUMN IF NOT EXISTS spells_known_progression_fixed INT[],
ADD COLUMN IF NOT EXISTS feat_progression JSONB;