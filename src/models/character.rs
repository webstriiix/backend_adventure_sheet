use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

impl Character {
    /// Clamp current_hp so it never exceeds max_hp.
    pub fn clamp_hp(&mut self) {
        self.current_hp = self.current_hp.min(self.max_hp);
    }

    /// Returns an error if any currency field is negative.
    pub fn validate_currency(&self) -> std::result::Result<(), String> {
        if self.cp < 0 {
            return Err("cp cannot be negative".into());
        }
        if self.sp < 0 {
            return Err("sp cannot be negative".into());
        }
        if self.ep < 0 {
            return Err("ep cannot be negative".into());
        }
        if self.gp < 0 {
            return Err("gp cannot be negative".into());
        }
        if self.pp < 0 {
            return Err("pp cannot be negative".into());
        }
        Ok(())
    }
}

impl AsiChoiceRequest {
    /// Validate ASI bumps against D&D 5e rules:
    /// - No single stat may be increased by more than 2.
    /// - If a feat is selected, no stat bumps are allowed.
    /// - Otherwise, you may increase one ability by 2 OR two abilities by 1 each.
    pub fn validate(&self) -> std::result::Result<(), String> {
        let bumps = [
            ("str", self.bump_str),
            ("dex", self.bump_dex),
            ("con", self.bump_con),
            ("int", self.bump_int),
            ("wis", self.bump_wis),
            ("cha", self.bump_cha),
        ];

        for (name, val) in &bumps {
            match val {
                Some(v) if *v > 2 => {
                    return Err(format!("Cannot increase {} by more than 2", name));
                }
                Some(v) if *v < 0 => {
                    return Err(format!("Cannot decrease {}", name));
                }
                _ => {}
            }
        }

        let total_bump: i32 = bumps.iter().map(|(_, v)| v.unwrap_or(0)).sum();
        let bumped_count = bumps.iter().filter(|(_, v)| v.unwrap_or(0) > 0).count();

        if self.feat_id.is_some() {
            if total_bump > 0 {
                return Err("Cannot increase ability scores when selecting a feat".into());
            }
            return Ok(());
        }

        match bumped_count {
            0 => {
                return Err("Must increase at least one ability score or select a feat".into());
            }
            1 => {
                if total_bump > 2 {
                    return Err("A single ability score can only be increased by up to 2".into());
                }
            }
            2 => {
                if total_bump > 2 {
                    return Err(
                        "When increasing two abilities, total increase cannot exceed 2".into(),
                    );
                }
            }
            _ => {
                return Err("Can only increase up to 2 ability scores".into());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Character {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub experience_pts: i32,
    pub race_id: Option<i32>,
    pub subrace_id: Option<i32>,
    pub background_id: Option<i32>,
    pub class_id: Option<i32>,
    pub str: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wis: i32,
    pub cha: i32,
    pub max_hp: i32,
    pub current_hp: i32,
    pub temp_hp: i32,
    pub inspiration: bool,
    pub notes: Option<String>,
    pub death_saves_successes: i32,
    pub death_saves_failures: i32,
    pub cp: i32,
    pub sp: i32,
    pub ep: i32,
    pub gp: i32,
    pub pp: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CharacterSpellSlot {
    pub character_id: Uuid,
    pub slot_level: i32,
    pub expended: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CharacterHitDice {
    pub character_id: Uuid,
    pub die_size: i32,
    pub expended: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateCharacter {
    pub name: String,
    pub class_id: i32,
    pub race_id: Option<i32>,
    pub subrace_id: Option<i32>,
    pub background_id: Option<i32>,
    pub str: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wis: i32,
    pub cha: i32,
    pub max_hp: i32,
    pub bonus_feat_id: Option<i32>,
    pub background_feat_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCharacter {
    pub name: String,
    pub class_id: Option<i32>,
    pub subclass_id: Option<i32>,
    pub experience_pts: i32,
    pub race_id: Option<i32>,
    pub subrace_id: Option<i32>,
    pub background_id: Option<i32>,
    pub str: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wis: i32,
    pub cha: i32,
    pub max_hp: i32,
    pub current_hp: i32,
    pub temp_hp: i32,
    pub inspiration: Option<bool>,
    pub notes: Option<String>,
    pub death_saves_successes: Option<i32>,
    pub death_saves_failures: Option<i32>,
    pub cp: Option<i32>,
    pub sp: Option<i32>,
    pub ep: Option<i32>,
    pub gp: Option<i32>,
    pub pp: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSpellSlot {
    pub expended: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHitDice {
    pub expended: i32,
}

#[derive(Debug, Deserialize)]
pub struct ShortRestRequest {
    pub hit_dice_spent: std::collections::HashMap<i32, i32>, // mapping die_size to amount spent
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CharacterClassInfo {
    pub class_id: i32,
    pub class_name: String,
    pub class_source: String,
    pub level: i32,
    pub is_primary: bool,
    pub subclass_id: Option<i32>,
    pub subclass_name: Option<String>,
    pub subclass_short_name: Option<String>,
    pub subclass_source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddClassRequest {
    pub class_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClassLevelRequest {
    pub level: i32,
    pub subclass_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AsiChoice {
    pub id: i64,
    pub character_id: Uuid,
    pub level: i32,
    pub bump_str: i32,
    pub bump_dex: i32,
    pub bump_con: i32,
    pub bump_int: i32,
    pub bump_wis: i32,
    pub bump_cha: i32,
    pub feat_id: Option<i32>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AsiChoiceRequest {
    pub bump_str: Option<i32>,
    pub bump_dex: Option<i32>,
    pub bump_con: Option<i32>,
    pub bump_int: Option<i32>,
    pub bump_wis: Option<i32>,
    pub bump_cha: Option<i32>,
    pub feat_id: Option<i32>,
    pub source_type: Option<String>,
    pub gained_at_level: Option<i32>,
}

// ── Progression Manifest ─────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WeaponMasteryChoice {
    pub id: i64,
    pub character_id: Uuid,
    pub mastery_name: String,
    pub source_level: i32,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ProgressionManifest {
    pub character_id: Uuid,
    pub total_level: i32,
    pub class_name: String,
    pub class_source: String,
    pub decision_points: Vec<DecisionPoint>,
}

#[derive(Debug, Serialize)]
pub struct DecisionPoint {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub level: i32,
    pub choice_type: String,
    pub required_count: i32,
    pub current_choices: Vec<ChoiceDetail>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ChoiceDetail {
    pub id: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── HP clamp ────────────────────────────────────────────────

    #[test]
    fn test_clamp_hp_does_nothing_when_under_max() {
        let mut c = Character {
            id: uuid::Uuid::nil(),
            user_id: uuid::Uuid::nil(),
            name: "test".into(),
            experience_pts: 0,
            race_id: None,
            subrace_id: None,
            background_id: None,
            class_id: None,
            str: 10,
            dex: 10,
            con: 10,
            int: 10,
            wis: 10,
            cha: 10,
            max_hp: 20,
            current_hp: 14,
            temp_hp: 0,
            inspiration: false,
            notes: None,
            death_saves_successes: 0,
            death_saves_failures: 0,
            cp: 0,
            sp: 0,
            ep: 0,
            gp: 0,
            pp: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        c.clamp_hp();
        assert_eq!(c.current_hp, 14);
    }

    #[test]
    fn test_clamp_hp_caps_at_max() {
        let mut c = Character {
            id: uuid::Uuid::nil(),
            user_id: uuid::Uuid::nil(),
            name: "test".into(),
            experience_pts: 0,
            race_id: None,
            subrace_id: None,
            background_id: None,
            class_id: None,
            str: 10,
            dex: 10,
            con: 10,
            int: 10,
            wis: 10,
            cha: 10,
            max_hp: 20,
            current_hp: 99,
            temp_hp: 0,
            inspiration: false,
            notes: None,
            death_saves_successes: 0,
            death_saves_failures: 0,
            cp: 0,
            sp: 0,
            ep: 0,
            gp: 0,
            pp: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        c.clamp_hp();
        assert_eq!(c.current_hp, 20);
    }

    #[test]
    fn test_clamp_hp_equal_is_fine() {
        let mut c = Character {
            id: uuid::Uuid::nil(),
            user_id: uuid::Uuid::nil(),
            name: "test".into(),
            experience_pts: 0,
            race_id: None,
            subrace_id: None,
            background_id: None,
            class_id: None,
            str: 10,
            dex: 10,
            con: 10,
            int: 10,
            wis: 10,
            cha: 10,
            max_hp: 20,
            current_hp: 20,
            temp_hp: 0,
            inspiration: false,
            notes: None,
            death_saves_successes: 0,
            death_saves_failures: 0,
            cp: 0,
            sp: 0,
            ep: 0,
            gp: 0,
            pp: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        c.clamp_hp();
        assert_eq!(c.current_hp, 20);
    }

    // ── Currency validation ─────────────────────────────────────

    #[test]
    fn test_currency_all_zero_is_ok() {
        let c = Character {
            id: uuid::Uuid::nil(),
            user_id: uuid::Uuid::nil(),
            name: "test".into(),
            experience_pts: 0,
            race_id: None,
            subrace_id: None,
            background_id: None,
            class_id: None,
            str: 10,
            dex: 10,
            con: 10,
            int: 10,
            wis: 10,
            cha: 10,
            max_hp: 20,
            current_hp: 20,
            temp_hp: 0,
            inspiration: false,
            notes: None,
            death_saves_successes: 0,
            death_saves_failures: 0,
            cp: 0,
            sp: 0,
            ep: 0,
            gp: 0,
            pp: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(c.validate_currency().is_ok());
    }

    #[test]
    fn test_currency_positive_is_ok() {
        let c = Character {
            id: uuid::Uuid::nil(),
            user_id: uuid::Uuid::nil(),
            name: "test".into(),
            experience_pts: 0,
            race_id: None,
            subrace_id: None,
            background_id: None,
            class_id: None,
            str: 10,
            dex: 10,
            con: 10,
            int: 10,
            wis: 10,
            cha: 10,
            max_hp: 20,
            current_hp: 20,
            temp_hp: 0,
            inspiration: false,
            notes: None,
            death_saves_successes: 0,
            death_saves_failures: 0,
            cp: 100,
            sp: 50,
            ep: 10,
            gp: 5,
            pp: 2,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(c.validate_currency().is_ok());
    }

    #[test]
    fn test_currency_negative_cp_fails() {
        let c = Character {
            id: uuid::Uuid::nil(),
            user_id: uuid::Uuid::nil(),
            name: "test".into(),
            experience_pts: 0,
            race_id: None,
            subrace_id: None,
            background_id: None,
            class_id: None,
            str: 10,
            dex: 10,
            con: 10,
            int: 10,
            wis: 10,
            cha: 10,
            max_hp: 20,
            current_hp: 20,
            temp_hp: 0,
            inspiration: false,
            notes: None,
            death_saves_successes: 0,
            death_saves_failures: 0,
            cp: -1,
            sp: 0,
            ep: 0,
            gp: 0,
            pp: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(c.validate_currency().is_err());
    }

    #[test]
    fn test_currency_negative_gp_fails() {
        let c = Character {
            id: uuid::Uuid::nil(),
            user_id: uuid::Uuid::nil(),
            name: "test".into(),
            experience_pts: 0,
            race_id: None,
            subrace_id: None,
            background_id: None,
            class_id: None,
            str: 10,
            dex: 10,
            con: 10,
            int: 10,
            wis: 10,
            cha: 10,
            max_hp: 20,
            current_hp: 20,
            temp_hp: 0,
            inspiration: false,
            notes: None,
            death_saves_successes: 0,
            death_saves_failures: 0,
            cp: 5,
            sp: 0,
            ep: 0,
            gp: -10,
            pp: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(c.validate_currency().is_err());
    }

    // ── ASI validation ──────────────────────────────────────────

    #[test]
    fn test_asi_single_stat_plus_2_is_ok() {
        let req = AsiChoiceRequest {
            bump_str: Some(2),
            bump_dex: None,
            bump_con: None,
            bump_int: None,
            bump_wis: None,
            bump_cha: None,
            feat_id: None,
            source_type: None,
            gained_at_level: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_asi_two_stats_plus_1_is_ok() {
        let req = AsiChoiceRequest {
            bump_str: Some(1),
            bump_dex: Some(1),
            bump_con: None,
            bump_int: None,
            bump_wis: None,
            bump_cha: None,
            feat_id: None,
            source_type: None,
            gained_at_level: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_asi_single_stat_plus_3_fails() {
        let req = AsiChoiceRequest {
            bump_str: Some(3),
            bump_dex: None,
            bump_con: None,
            bump_int: None,
            bump_wis: None,
            bump_cha: None,
            feat_id: None,
            source_type: None,
            gained_at_level: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_asi_three_stats_fails() {
        let req = AsiChoiceRequest {
            bump_str: Some(1),
            bump_dex: Some(1),
            bump_con: Some(1),
            bump_int: None,
            bump_wis: None,
            bump_cha: None,
            feat_id: None,
            source_type: None,
            gained_at_level: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_asi_with_feat_and_bumps_fails() {
        let req = AsiChoiceRequest {
            bump_str: Some(1),
            bump_dex: None,
            bump_con: None,
            bump_int: None,
            bump_wis: None,
            bump_cha: None,
            feat_id: Some(42),
            source_type: None,
            gained_at_level: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_asi_feat_only_is_ok() {
        let req = AsiChoiceRequest {
            bump_str: None,
            bump_dex: None,
            bump_con: None,
            bump_int: None,
            bump_wis: None,
            bump_cha: None,
            feat_id: Some(42),
            source_type: None,
            gained_at_level: None,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_asi_negative_bump_fails() {
        let req = AsiChoiceRequest {
            bump_str: Some(-1),
            bump_dex: None,
            bump_con: None,
            bump_int: None,
            bump_wis: None,
            bump_cha: None,
            feat_id: None,
            source_type: None,
            gained_at_level: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_asi_no_bumps_and_no_feat_fails() {
        let req = AsiChoiceRequest {
            bump_str: None,
            bump_dex: None,
            bump_con: None,
            bump_int: None,
            bump_wis: None,
            bump_cha: None,
            feat_id: None,
            source_type: None,
            gained_at_level: None,
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_asi_total_3_with_two_stats_fails() {
        let req = AsiChoiceRequest {
            bump_str: Some(2),
            bump_dex: Some(1),
            bump_con: None,
            bump_int: None,
            bump_wis: None,
            bump_cha: None,
            feat_id: None,
            source_type: None,
            gained_at_level: None,
        };
        assert!(req.validate().is_err());
    }
}
