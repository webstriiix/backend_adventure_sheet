use anyhow::Result;
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct HomebrewClassInput {
    pub name: String,
    pub source_slug: String,
    pub source_name: String,
    pub description: String, // Teks mentah dari user
    pub hit_die: u8,
    pub primary_ability: Vec<String>, // ["intelligence", "dexterity"]
    pub saves: Vec<String>,           // ["intelligence", "wisdom"]
    pub armor: Vec<String>,           // ["light", "medium"]
    pub weapons: Vec<String>,         // ["simple", "rapier", "shortsword"]
    pub skills: Vec<String>,          // ["history", "insight", "investigation", ...]
    pub skill_choices: u8,            // Berapa skill yang dipilih
    pub subclass_level: u8,           // Level subclass (default 3)
    pub subclass_title: String,       // "Tactician Subclass"
    pub features_text: String,        // Teks fitur per level (parsing manual/heuristic)
    pub spellcasting: Option<SpellcastingInfo>,
}

#[derive(Debug, Clone)]
pub struct SpellcastingInfo {
    pub ability: String,
    pub progression: String, // "full", "1/2", "1/3", "pact", "none"
    pub ritual: bool,
    pub focus: Option<String>,
}

pub fn parse_homebrew_class_to_json(input: HomebrewClassInput) -> Result<Value> {
    // TODO: Implementasi parsing heuristik dari teks ke struktur 5etools
    // Gunakan regex, keyword matching, dan aturan D&D 2024 untuk ekstrak:
    // - class table (level, features, prof bonus, spell slots)
    // - features per level (name, level, entries)
    // - subclass features
    // - spellcasting progression
    // - starting equipment
    // - skill choices

    // Contoh output minimal:
    Ok(json!({
        "name": input.name,
        "source": input.source_slug,
        "page": 1,
        "srd": false,
        "basicRules": false,
        "edition": "one", // 2024
        "hd": { "number": 1, "faces": input.hit_die as i32 },
        "proficiency": input.saves.iter().map(|s| s.to_lowercase()).collect::<Vec<_>>(),
        "spellcastingAbility": input.spellcasting.as_ref().map(|s| s.ability.clone()),
        "casterProgression": input.spellcasting.as_ref().map(|s| s.progression.clone()),
        "weaponProficiencies": input.weapons,
        "armorProficiencies": input.armor,
        "skillChoices": [{
            "choose": input.skill_choices,
            "from": input.skills,
            "type": "skill"
        }],
        "startingEquipment": json!({ "defaultData": [] }), // TODO: parse dari teks
        "multiclassRequirements": null,
        "classTableGroups": [], // TODO: parse dari features_text
        "subclassTitle": input.subclass_title,
        "features": [], // TODO: parse dari features_text
    }))
}
