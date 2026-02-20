-- ============================================================
-- CLASSES  (Wizard/PHB, Druid/PHB, etc.)
-- ============================================================
CREATE TABLE classes (
    id                      SERIAL PRIMARY KEY,
    name                    TEXT NOT NULL,
    source_id               INT  NOT NULL REFERENCES sources(id),
    hit_die                 INT  NOT NULL,
    proficiency_saves       TEXT[] NOT NULL DEFAULT '{}',
    spellcasting_ability    TEXT,
    caster_progression      TEXT,
    weapon_proficiencies    TEXT[] NOT NULL DEFAULT '{}',
    armor_proficiencies     TEXT[] NOT NULL DEFAULT '{}',
    skill_choices           JSONB NOT NULL DEFAULT '{}',
    starting_equipment      JSONB NOT NULL DEFAULT '{}',
    multiclass_requirements JSONB,
    class_table             JSONB NOT NULL DEFAULT '[]',
    subclass_title          TEXT NOT NULL DEFAULT 'Subclass',
    edition                 TEXT,
    UNIQUE(name, source_id)
);
