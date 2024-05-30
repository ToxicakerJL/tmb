use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use serde::Deserialize;
use crate::utils::get_project_root_path;

pub const ENCOUNTER_CARD_GENERAL_PATH: &str = "/config/encounters/general";
pub const TYRANT_CARD_GENERAL_PATH: &str = "/config/tyrants";

#[derive(Debug, Deserialize)]
pub struct EncounterCard {
    pub title: String,
    pub story: String,
    pub choices: Vec<Choice>,
    pub remark: String,
    pub progress: usize,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub description: String,
    pub action: String,
    pub rewards: String,
}

#[derive(Debug, Deserialize)]
pub struct TyrantCard {
    pub name: String,
    pub description: String,
    pub min_days: usize,
    pub max_days: usize,
    pub game_length: String,
    pub battle_title: String,
    pub creatures: String,
    pub battle_mechanism: Vec<String>,
    pub tyrant_skills: Vec<String>,
    pub tyrant_die: Vec<String>,
}

impl EncounterCard {
    pub fn new(file_path: &str) -> Option<EncounterCard> {
        let mut file = File::open(file_path).expect(format!("Failed to open file {}", file_path).as_str());
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(format!("Failed to read file {}", file_path).as_str());
        let encounter_card: EncounterCard = serde_yaml::from_str(&contents).expect(format!("Failed to parse file {}", file_path).as_str());
        Some(encounter_card)
    }
}

impl TyrantCard {
    pub fn new(file_path: &str) -> Option<TyrantCard> {
        let mut file = File::open(file_path).expect(format!("Failed to open file {}", file_path).as_str());
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(format!("Failed to read file {}", file_path).as_str());
        let tyrant_card: TyrantCard = serde_yaml::from_str(&contents).expect(format!("Failed to parse file {}", file_path).as_str());
        Some(tyrant_card)
    }

    pub fn list() -> Vec<TyrantCard> {
        let dir_path = get_project_root_path() + TYRANT_CARD_GENERAL_PATH;
        let mut tyrant_cards: Vec<TyrantCard> = Vec::new();
        for file in std::fs::read_dir(PathBuf::from(dir_path.as_str())).expect(format!("Failed to open directory {}", dir_path.as_str()).as_str()) {
            let path = file.unwrap().path();
            if path.is_file() {
                let card = TyrantCard::new(path.to_str().unwrap());
                match card {
                    Some(c) => tyrant_cards.push(c),
                    _ => {}
                }
            }
        }
        tyrant_cards
    }
}