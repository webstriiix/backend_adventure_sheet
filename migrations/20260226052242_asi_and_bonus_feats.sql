-- Add asi_levels to classes table
ALTER TABLE classes 
ADD COLUMN asi_levels integer[] DEFAULT '{}';

-- Add grants_bonus_feat to races table
ALTER TABLE races 
ADD COLUMN grants_bonus_feat boolean DEFAULT false;
