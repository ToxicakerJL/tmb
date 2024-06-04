use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Mutex;
use lazy_static::lazy_static;
use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;
use serde::Deserialize;
use tracing::info;
use crate::utils::get_project_root_path;

/// Some global variables about some basic game information:
/// 1. Encounter cards.
/// 2. Tyrant cards.
/// 3. The encounter deck built during runtime.
///
/// These global card deck maintain just one copy and should be globally used.
/// `EncounterDeck` has basic deck build strategies and shuffle strategies.
static ENCOUNTER_CARD_GENERAL_PATH: &str = "/config/encounters/general";
static ENCOUNTER_CARD_DAY1_PATH: &str = "/config/encounters/day1";
static ENCOUNTER_CARD_DAY2_PATH: &str = "/config/encounters/day2";
static ENCOUNTER_CARD_DAY3_PATH: &str = "/config/encounters/day3";
static ENCOUNTER_CARD_SPECIAL_PATH: &str = "/config/encounters/special";
static ENCOUNTER_CARD_TYRANT_PATH: &str = "/config/encounters/tyrant";
static TYRANT_CARD_GENERAL_PATH: &str = "/config/tyrants";

pub static TYRANT_NAME_DUSTER: &str = "Duster";
pub static TYRANT_NAME_NOM: &str = "Nom";
pub static TYRANT_NAME_DRELLEN: &str = "Drellen";
pub static TYRANT_NAME_MULMESH: &str = "Mulmesh";
pub static TYRANT_NAME_GENDRICKS: &str = "Gendricks";
pub static TYRANT_NAME_GOBLIN_KING: &str = "Goblin_King";
pub static TYRANT_NAME_MARROW: &str = "Marrow";

lazy_static! {
    pub static ref DAY1_ENCOUNTER_CARDS: Mutex<Vec<EncounterCard>> = Mutex::new(list_day1_encounter_cards());
    pub static ref DAY2_ENCOUNTER_CARDS: Mutex<Vec<EncounterCard>> = Mutex::new(list_day2_encounter_cards());
    pub static ref DAY3_ENCOUNTER_CARDS: Mutex<Vec<EncounterCard>> = Mutex::new(list_day3_encounter_cards());
    pub static ref SPECIAL_ENCOUNTER_CARDS: Mutex<Vec<EncounterCard>> = Mutex::new(list_special_encounter_cards());
    pub static ref GENERAL_ENCOUNTER_CARDS: Mutex<Vec<EncounterCard>> = Mutex::new(list_general_encounter_cards());
    pub static ref TYRANT_ENCOUNTER_CARDS: Mutex<HashMap<String, Vec<EncounterCard>>> = Mutex::new(list_tyrant_encounter_cards());
    pub static ref TYRANT_CARDS: Mutex<Vec<TyrantCard>> = Mutex::new(list_tyrant_cards());
}

pub fn list_day1_encounter_cards() -> Vec<EncounterCard> {
    list_encounter_cards(get_project_root_path() + ENCOUNTER_CARD_DAY1_PATH, None)
}

pub fn list_day2_encounter_cards() -> Vec<EncounterCard> {
    list_encounter_cards(get_project_root_path() + ENCOUNTER_CARD_DAY2_PATH, None)
}

pub fn list_day3_encounter_cards() -> Vec<EncounterCard> {
    list_encounter_cards(get_project_root_path() + ENCOUNTER_CARD_DAY3_PATH, None)
}

pub fn list_general_encounter_cards() -> Vec<EncounterCard> {
    list_encounter_cards(get_project_root_path() + ENCOUNTER_CARD_GENERAL_PATH, None)
}

pub fn list_special_encounter_cards() -> Vec<EncounterCard> {
    list_encounter_cards(get_project_root_path() + ENCOUNTER_CARD_SPECIAL_PATH, None)
}

pub fn list_tyrant_encounter_cards() -> HashMap<String, Vec<EncounterCard>> {
    let tyrant_names = [TYRANT_NAME_DUSTER, TYRANT_NAME_NOM, TYRANT_NAME_GENDRICKS, TYRANT_NAME_DRELLEN, TYRANT_NAME_MARROW, TYRANT_NAME_GOBLIN_KING, TYRANT_NAME_MULMESH];
    let mut map = HashMap::new();
    for name in tyrant_names {
        map.insert(name.to_lowercase(), list_encounter_cards(get_project_root_path() + ENCOUNTER_CARD_TYRANT_PATH, Some(name.to_lowercase())));
    }
    map
}

pub fn list_tyrant_cards() -> Vec<TyrantCard> {
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

fn list_encounter_cards(dir_path: String, file_name_filter: Option<String>) -> Vec<EncounterCard> {
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

#[derive(Debug, Deserialize, Clone)]
pub struct EncounterCard {
    pub title: String,
    pub story: String,
    pub choices: Vec<Choice>,
    pub remark: String,
    pub progress: Vec<usize>,
    pub card_type: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Choice {
    pub description: String,
    pub action: String,
    pub rewards: String,
}

#[derive(Debug, Deserialize)]
pub struct TyrantCard {
    pub id: String,
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
}

impl TyrantCard {
    pub fn new(file_path: &str) -> Option<TyrantCard> {
        let mut file = File::open(file_path).expect(format!("Failed to open file {}", file_path).as_str());
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(format!("Failed to read file {}", file_path).as_str());
        let tyrant_card: TyrantCard = serde_yaml::from_str(&contents).expect(format!("Failed to parse file {}", file_path).as_str());
        Some(tyrant_card)
    }
}

#[derive(Debug)]
pub struct EncounterDeck {
    pub tyrant_card: TyrantCard,
    pub encounter_cards: Vec<EncounterCard>,
}

impl EncounterDeck {
    pub fn new(tyrant_name: &str) -> Self {
        let mut encounter_cards: Vec<EncounterCard> = Vec::new();
        let mut rng = thread_rng();
        let file_path = get_project_root_path() + TYRANT_CARD_GENERAL_PATH + "/" + tyrant_name + ".yaml";
        let tyrant_card = TyrantCard::new(file_path.as_str()).expect(format!("Tyrant {} doesn't exist", tyrant_name).as_str());
        info!("Building encounter decks with selected tyrant card {:?}......", tyrant_card);
        // Build tyrant encounter cards. Remove the cards from global deck.
        let mut tyrant_encounter_card_map = TYRANT_ENCOUNTER_CARDS.lock().unwrap();
        let tyrant_encounter_cards = tyrant_encounter_card_map.get(tyrant_name).unwrap();
        let size = tyrant_card.max_days - tyrant_encounter_cards.len() - 3;
        let pickup = tyrant_encounter_card_map.remove(tyrant_name).unwrap();
        info!("Selected tyrant encounter cards {:?}......", pickup);
        encounter_cards.extend(pickup);

        // Build general encounter cards. Remove the cards from global deck.
        let mut general_encounter_cards = GENERAL_ENCOUNTER_CARDS.lock().unwrap();
        for _ in 0..size {
            let rand = rng.gen_range(0..general_encounter_cards.len());
            let pickup = general_encounter_cards.remove(rand);
            info!("Selected general encounter cards {:?}......", pickup);
            encounter_cards.push(pickup);
        }

        encounter_cards.shuffle(&mut rng);

        // Build day1, day2, day3 encounter cards. Remove the cards from global deck.
        let mut day1_cards = DAY1_ENCOUNTER_CARDS.lock().unwrap();
        let day1_rand = rng.gen_range(0..day1_cards.len());
        let day1_card = day1_cards.remove(day1_rand);
        info!("Selected day1 encounter card {:?}......", day1_card);
        encounter_cards.insert(0, day1_card);

        let mut day2_cards = DAY2_ENCOUNTER_CARDS.lock().unwrap();
        let day2_rand = rng.gen_range(0..day2_cards.len());
        let day2_card = day2_cards.remove(day2_rand);
        info!("Selected day2 encounter card {:?}......", day2_card);
        encounter_cards.insert(1, day2_card);

        let mut day3_cards = DAY3_ENCOUNTER_CARDS.lock().unwrap();
        let day3_rand = rng.gen_range(0..day3_cards.len());
        let day3_card = day3_cards.remove(day3_rand);
        info!("Selected day3 encounter card {:?}......", day3_card);
        encounter_cards.insert(2, day3_card);
        let deck = EncounterDeck {
            tyrant_card,
            encounter_cards,
        };
        info!("Encounter deck built: {:?}", deck);
        deck
    }

    pub fn shuffle(&mut self, shuffle_strategy: ShuffleStrategy, encounter_card: Option<EncounterCard>) {
        let mut rng = thread_rng();
        match shuffle_strategy {
            ShuffleStrategy::PutCurrentCardTop => {
                info!("[{:?}] Put {:?} on deck top. Current deck: {:?}", shuffle_strategy, encounter_card, self.encounter_cards);
                self.encounter_cards.insert(0, encounter_card.expect("Expect the current encounter card!"));
            }
            ShuffleStrategy::PutCurrentCardRandom => {
                info!("[{:?}] Shuffled {:?} into the encounter deck. Current deck: {:?}", shuffle_strategy, encounter_card, self.encounter_cards);
                let idx = rng.gen_range(0..self.encounter_cards.len());
                self.encounter_cards.insert(idx, encounter_card.expect("Expect the current encounter card!"));
            }
            ShuffleStrategy::FirstTyrantCardTopAndShuffleRest => {
                let mut i = 0;
                while i < self.encounter_cards.len() {
                    if let Some(card) = self.encounter_cards.get(i) {
                        if card.card_type == "tyrant" {
                            break;
                        }
                    }
                    i += 1;
                }
                if i < self.encounter_cards.len() {
                    let card = self.encounter_cards.remove(i);
                    info!("[{:?}] Put {:?} on the encounter deck top. Current deck: {:?}", shuffle_strategy, encounter_card, self.encounter_cards);
                    self.encounter_cards.shuffle(&mut rng);
                    self.encounter_cards.insert(0, card);
                }
            }
            ShuffleStrategy::PickSpecialCardAndShuffle => {
                info!("[{:?}] Shuffled {:?} into the encounter deck. Current deck: {:?}", shuffle_strategy, encounter_card, self.encounter_cards);
                self.encounter_cards.push(encounter_card.expect("Expect the special encounter card!"));
                let mut idx = 0;
                while idx < self.encounter_cards.len() {
                    if !&self.encounter_cards[idx].card_type.contains("day") {
                        break;
                    }
                    idx += 1;
                }
                self.encounter_cards[idx..].shuffle(&mut rng);
            }
            ShuffleStrategy::ReplaceTodayEncounterAndShuffleTodayEncounter => {
                let mut general_cards = GENERAL_ENCOUNTER_CARDS.lock().unwrap();
                let rand1 = rng.gen_range(1..general_cards.len());
                let rand2 = rng.gen_range(0..self.encounter_cards.len());
                let replacement = general_cards.remove(rand1);
                info!("[{:?}] Replaced today's encounter card with {:?}. Shuffled {:?} into the encounter deck. Current deck: {:?}", shuffle_strategy, replacement, encounter_card, self.encounter_cards);
                self.encounter_cards.insert(rand2, encounter_card.expect("Expect the current encounter card!"));
                self.encounter_cards.insert(0, replacement);
            }
        }
    }

    pub fn rollback(&mut self, finished_encounter_cards: &mut Vec<EncounterCard>, day: usize) {
        if day <= finished_encounter_cards.len() {
            info!("Started rolling back encounter cards to day {}", day);
            let mut i = finished_encounter_cards.len();
            while i >= day {
                let card = finished_encounter_cards.pop().unwrap();
                info!("Insert {} day card: {:?}", i, card);
                self.encounter_cards.insert(0, card);
                i -= 1;
            }
            info!("Finished encounter deck rollback. Current encounter deck: {:?}", self.encounter_cards);
        }
    }
}

#[derive(Debug)]
pub enum ShuffleStrategy {
    PutCurrentCardTop,
    PutCurrentCardRandom,
    FirstTyrantCardTopAndShuffleRest,
    PickSpecialCardAndShuffle,
    ReplaceTodayEncounterAndShuffleTodayEncounter,
}