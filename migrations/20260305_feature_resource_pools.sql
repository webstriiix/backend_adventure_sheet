CREATE TABLE character_resource_pools (
    character_id UUID NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    resource_name TEXT NOT NULL,
    uses_remaining INT NOT NULL,
    PRIMARY KEY (character_id, resource_name)
);
