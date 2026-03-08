-- Add columns for background features
ALTER TABLE backgrounds
ADD COLUMN ability_bonuses jsonb,
ADD COLUMN grants_bonus_feat boolean DEFAULT false;
