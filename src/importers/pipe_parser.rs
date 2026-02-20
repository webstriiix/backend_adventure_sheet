/// Parses: "FeatureName|ClassName|ClassSource|Level"
/// or:     "FeatureName|ClassName|ClassSource|SubclassShortName|SubclassSource|Level"
#[derive(Debug)]
pub struct FeatureRef {
    pub name:                 String,
    pub class_name:           String,
    pub class_source:         String,         // "" means default to PHB
    pub subclass_short_name:  Option<String>,
    pub subclass_source:      Option<String>,
    pub level:                u8,
    pub extra_source:         Option<String>, // trailing source like TCE
}

pub fn parse_feature_ref(raw: &str) -> Option<FeatureRef> {
    let parts: Vec<&str> = raw.split('|').collect();
    match parts.len() {
        // "Name|ClassName||Level"  or  "Name|ClassName||Level|ExtraSource"
        4 | 5 => Some(FeatureRef {
            name:                parts[0].to_string(),
            class_name:          parts[1].to_string(),
            class_source:        if parts[2].is_empty() { "PHB".into() } else { parts[2].to_string() },
            subclass_short_name: None,
            subclass_source:     None,
            level:               parts[3].parse().unwrap_or(0),
            extra_source:        parts.get(4).map(|s| s.to_string()),
        }),
        // "Name|ClassName|ClassSource|SubclassShortName|SubclassSource|Level"
        6 => Some(FeatureRef {
            name:                parts[0].to_string(),
            class_name:          parts[1].to_string(),
            class_source:        if parts[2].is_empty() { "PHB".into() } else { parts[2].to_string() },
            subclass_short_name: Some(parts[3].to_string()),
            subclass_source:     Some(if parts[4].is_empty() { "PHB".into() } else { parts[4].to_string() }),
            level:               parts[5].parse().unwrap_or(0),
            extra_source:        None,
        }),
        _ => None,
    }
}

/// Parses class_features array entries which are either:
/// - A plain string:  "Arcane Recovery|Wizard||1"
/// - An object:       {"classFeature": "...", "gainSubclassFeature": true}
#[derive(Debug)]
pub struct ClassFeatureEntry {
    pub feature_ref:       FeatureRef,
    pub gain_subclass:     bool,
}

pub fn parse_class_feature_entry(value: &serde_json::Value) -> Option<ClassFeatureEntry> {
    match value {
        serde_json::Value::String(s) => {
            parse_feature_ref(s).map(|r| ClassFeatureEntry {
                feature_ref:   r,
                gain_subclass: false,
            })
        }
        serde_json::Value::Object(obj) => {
            let raw = obj.get("classFeature")?.as_str()?;
            let gain = obj.get("gainSubclassFeature")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            parse_feature_ref(raw).map(|r| ClassFeatureEntry {
                feature_ref:   r,
                gain_subclass: gain,
            })
        }
        _ => None,
    }
}
