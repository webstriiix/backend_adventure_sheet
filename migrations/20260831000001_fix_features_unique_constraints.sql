-- Drop old constraints that didn't include level
ALTER TABLE class_features DROP CONSTRAINT IF EXISTS class_features_name_source_id_class_id_key;
ALTER TABLE subclass_features DROP CONSTRAINT IF EXISTS subclass_features_name_source_id_subclass_id_key;

-- Drop new constraints if already created
ALTER TABLE class_features DROP CONSTRAINT IF EXISTS class_features_class_id_source_id_name_level_key;
ALTER TABLE subclass_features DROP CONSTRAINT IF EXISTS subclass_features_subclass_id_source_id_name_level_key;

-- Add new unique constraints that include level
ALTER TABLE class_features ADD CONSTRAINT class_features_class_id_source_id_name_level_key UNIQUE (class_id, source_id, name, level);
ALTER TABLE subclass_features ADD CONSTRAINT subclass_features_subclass_id_source_id_name_level_key UNIQUE (subclass_id, source_id, name, level);
