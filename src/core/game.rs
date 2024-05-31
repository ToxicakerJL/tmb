use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;
use serde::Deserialize;
use crate::utils::get_project_root_path;

const ENCOUNTER_CARD_GENERAL_PATH: &str = "/config/encounters/general";
const ENCOUNTER_CARD_DAY1_PATH: &str = "/config/encounters/day1";
const ENCOUNTER_CARD_DAY2_PATH: &str = "/config/encounters/day2";
const ENCOUNTER_CARD_DAY3_PATH: &str = "/config/encounters/day3";
const ENCOUNTER_CARD_SPECIAL_PATH: &str = "/config/encounters/special";
const ENCOUNTER_CARD_TYRANT_PATH: &str = "/config/encounters/tyrant";
const TYRANT_CARD_GENERAL_PATH: &str = "/config/tyrants";

#[derive(Debug, Deserialize, Clone)]
pub struct EncounterCard {
    pub title: String,
    pub story: String,
    pub choices: Vec<Choice>,
    pub remark: String,
    pub progress: Vec<usize>,
    pub card_type: String
}

#[derive(Debug, Deserialize, Clone)]
pub struct Choice {
    pub description: String,
    pub action: String,
    pub rewards: String,
}

#[derive(Debug, Deserialize)]
pub struct TyrantCard {
    pub name: String,
    pub description: String,
    pub min_progress: usize,
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

    pub fn list_day1_cards() -> Vec<EncounterCard> {
        Self::list(get_project_root_path() + ENCOUNTER_CARD_DAY1_PATH, None)
    }

    pub fn list_day2_cards() -> Vec<EncounterCard> {
        Self::list(get_project_root_path() + ENCOUNTER_CARD_DAY2_PATH, None)
    }

    pub fn list_day3_cards() -> Vec<EncounterCard> {
        Self::list(get_project_root_path() + ENCOUNTER_CARD_DAY3_PATH, None)
    }

    pub fn list_general_cards() -> Vec<EncounterCard> {
        Self::list(get_project_root_path() + ENCOUNTER_CARD_GENERAL_PATH, None)
    }

    pub fn list_special_cards() -> Vec<EncounterCard> {
        Self::list(get_project_root_path() + ENCOUNTER_CARD_SPECIAL_PATH, None)
    }

    pub fn list_tyrant_encounter_cards(tyrant_name: &str) -> Vec<EncounterCard> {
        Self::list(get_project_root_path() + ENCOUNTER_CARD_TYRANT_PATH, Some(tyrant_name.to_string()))
    }

    fn list(dir_path: String, file_name_filter: Option<String>) -> Vec<EncounterCard> {
        let mut encounter_cards: Vec<EncounterCard> = Vec::new();
        for file in std::fs::read_dir(PathBuf::from(dir_path.as_str())).expect(format!("Failed to open directory {}", dir_path.as_str()).as_str()) {
            let path = file.unwrap().path();
            if path.is_file() {
                if let Some(ref filter) = file_name_filter {
                    if path.file_name().unwrap().to_string_lossy().to_string().contains(filter) {
                        let card = EncounterCard::new(path.to_str().unwrap());
                        match card {
                            Some(c) => encounter_cards.push(c),
                            _ => {}
                        }
                    }
                } else {
                    let card = EncounterCard::new(path.to_str().unwrap());
                    match card {
                        Some(c) => encounter_cards.push(c),
                        _ => {}
                    }
                }
            }
        }
        encounter_cards
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

#[derive(Debug)]
pub struct EncounterDeck {
    pub tyrant_card: TyrantCard,
    pub encounter_cards: Vec<EncounterCard>,
}

impl EncounterDeck {
    pub fn new(tyrant_name: &str) -> Self {
        let mut encounter_cards = Vec::new();
        let mut rng = thread_rng();
        let file_path = get_project_root_path() + TYRANT_CARD_GENERAL_PATH + "/" + tyrant_name + ".yaml";
        let tyrant_card = TyrantCard::new(file_path.as_str()).expect(format!("Tyrant {} doesn't exist", tyrant_name).as_str());

        let tyrant_encounter_cards = EncounterCard::list_tyrant_encounter_cards(tyrant_name);
        let size = tyrant_card.max_days - tyrant_encounter_cards.len() - 3;
        encounter_cards.extend(tyrant_encounter_cards);

        let mut general_encounter_cards = EncounterCard::list_general_cards();
        for _ in 0..size {
            let pickup = general_encounter_cards.remove(rng.gen_range(0..general_encounter_cards.len()));
            encounter_cards.push(pickup);
        }
        let mut rng = thread_rng();
        encounter_cards.shuffle(&mut rng);

        let mut day1_cards = EncounterCard::list_day1_cards();
        encounter_cards.insert(0, day1_cards.remove(rng.gen_range(0..day1_cards.len())));
        let mut day2_cards = EncounterCard::list_day2_cards();
        encounter_cards.insert(1, day2_cards.remove(rng.gen_range(0..day2_cards.len())));
        let mut day3_cards = EncounterCard::list_day3_cards();
        encounter_cards.insert(2, day3_cards.remove(rng.gen_range(0..day3_cards.len())));
        EncounterDeck {
            tyrant_card,
            encounter_cards,
        }
    }

    pub fn shuffle(&mut self, shuffle_strategy: ShuffleStrategy, encounter_card: Option<EncounterCard>) {
        let mut rng = thread_rng();
        match shuffle_strategy {
            ShuffleStrategy::PutCurrentCardTop => {
                self.encounter_cards.insert(0, encounter_card.expect("Expect the current encounter card!"));
            }
            ShuffleStrategy::PutCurrentCardRandom => {
                let idx = rng.gen_range(0..self.encounter_cards.len());
                self.encounter_cards.insert(idx, encounter_card.expect("Expect the current encounter card!"));
            }
            ShuffleStrategy::FirstTyrantCardTopAndShuffleRest => {
                let mut i = 0;
                while i < self.encounter_cards.len() {
                    if let Some(card) =  self.encounter_cards.get(i) {
                        if card.card_type == "tyrant" {
                            break;
                        }
                    }
                    i += 1;
                }
                let card = self.encounter_cards.remove(i);
                self.encounter_cards.shuffle(&mut rng);
                self.encounter_cards.insert(0, card);
            }
            ShuffleStrategy::PickSpecialCardAndShuffle => {
                self.encounter_cards.push(encounter_card.expect("Expect the special encounter card!"));
                self.encounter_cards.shuffle(&mut rng);
            }
        }
    }
}

pub enum ShuffleStrategy {
    PutCurrentCardTop,
    PutCurrentCardRandom,
    FirstTyrantCardTopAndShuffleRest,
    PickSpecialCardAndShuffle,
}