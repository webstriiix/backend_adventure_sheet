-- Add mastery column to items table
ALTER TABLE items ADD COLUMN mastery TEXT[] NOT NULL DEFAULT '{}';
