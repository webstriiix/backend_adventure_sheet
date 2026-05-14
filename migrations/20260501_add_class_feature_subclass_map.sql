-- Create mapping table between class_features (gates) and subclass_features (content unlocked by gate)
CREATE TABLE class_feature_subclass_map (
    id SERIAL PRIMARY KEY,
    class_feature_id INT NOT NULL REFERENCES class_features(id) ON DELETE CASCADE,
    subclass_feature_id INT NOT NULL REFERENCES subclass_features(id) ON DELETE CASCADE,
    UNIQUE(class_feature_id, subclass_feature_id)
);

CREATE INDEX idx_cfsm_class_feature_id ON class_feature_subclass_map(class_feature_id);
CREATE INDEX idx_cfsm_subclass_feature_id ON class_feature_subclass_map(subclass_feature_id);
