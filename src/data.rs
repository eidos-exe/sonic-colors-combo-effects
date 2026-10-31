use std::{collections::HashMap, fs};
use serde::Deserialize;

fn combo_frame() -> i32 { 90 }
fn cooldown_frame() -> i32 { 16 }
fn offset_x() -> f32 { 8.0 }
fn offset_y() -> f32 { 12.0 }
fn effect_scale() -> f32 { 1.2 }

#[derive(Debug, Deserialize, Clone)]
pub struct ComboRank {
    pub hits: i32,
    pub sound: String,
    pub effect: String,
    #[serde(default = "effect_scale")]
    pub scale: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CharacterData {
    #[serde(default = "combo_frame")]
    pub combo_frame: i32,
    #[serde(default = "cooldown_frame")]
    pub cooldown_frame: i32,
    #[serde(default = "offset_x")]
    pub offset_x: f32,
    #[serde(default = "offset_y")]
    pub offset_y: f32,
    pub ranks: Vec<ComboRank>,
}

impl CharacterData {
    pub fn rank_for_hits(&self, hits: i32) -> Option<&ComboRank> {
        self.ranks.iter().rev().find(|rank| hits >= rank.hits)
    }
}

pub fn parse_effect_config(content: &str) -> Result<HashMap<String, CharacterData>, String> {
    let raw: HashMap<String, Vec<CharacterData>> = toml::from_str(content)
        .map_err(|error| format!("Invalid combo config: {}", error))?;
    let mut config = HashMap::new();
    for (name, mut entries) in raw {
        if entries.len() != 1 {
            return Err(format!("Expected one config for {}", name));
        }
        let entry = entries.remove(0);
        if entry.combo_frame <= 0 || entry.cooldown_frame < 0 || entry.ranks.is_empty()
            || !entry.offset_x.is_finite() || !entry.offset_y.is_finite() {
            return Err(format!("Invalid combo settings for {}", name));
        }
        let mut previous_hits = 0;
        for rank in &entry.ranks {
            if rank.hits <= previous_hits || rank.sound.is_empty() || rank.effect.is_empty()
                || !rank.scale.is_finite() || rank.scale <= 0.0 {
                return Err(format!("Invalid rank settings for {}", name));
            }
            previous_hits = rank.hits;
        }
        config.insert(name, entry);
    }
    Ok(config)
}

pub fn load_effect_config(path: &str) -> HashMap<String, CharacterData> {
    match fs::read_to_string(path).map_err(|error| error.to_string())
        .and_then(|content| parse_effect_config(&content)) {
        Ok(config) => config,
        Err(error) => {
            println!("[sonic_combo_fx] {}", error);
            HashMap::new()
        }
    }
}
