-- Remove the unique constraint on (character_id, feat_id)
ALTER TABLE character_feats DROP CONSTRAINT character_feats_character_id_feat_id_key;
