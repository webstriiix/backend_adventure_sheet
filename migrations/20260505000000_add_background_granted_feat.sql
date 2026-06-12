-- Link backgrounds to specific feats instead of just a boolean flag
ALTER TABLE backgrounds
ADD COLUMN granted_feat_id INT REFERENCES feats(id);
