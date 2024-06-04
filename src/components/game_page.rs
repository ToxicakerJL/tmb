use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::*;
use ratatui::widgets::block::{Position, Title};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
use tui_big_text::{BigText, PixelSize};
use crate::app::Action;
use crate::component::Component;
use crate::components::game_page::ShowPopup::{BreakPopup, ChallengeSuccessfulPopup, NoPopUp};
use crate::components::popup::Popup;
use crate::core::game::{Choice, EncounterCard, EncounterDeck, SPECIAL_ENCOUNTER_CARDS};
use crate::core::game::ShuffleStrategy::{FirstTyrantCardTopAndShuffleRest, PickSpecialCardAndShuffle, PutCurrentCardRandom, PutCurrentCardTop, ReplaceTodayEncounterAndShuffleTodayEncounter};
use crate::utils::centered_rect;

pub const NAME: &str = "GamePage";

pub struct GamePage {
    pub name: String,
    pub action_sender: Option<UnboundedSender<Action>>,
    deck: Option<EncounterDeck>,    // Built encounter deck
    days: usize,                    // The current day
    progress: usize,                // The current progress
    today_card: Option<EncounterCard>,
    should_go_next_day: bool,
    finished_encounter_cards: Vec<EncounterCard>, // The selected choice for the current day
    popup: ShowPopup, // show the popup dialog
    selected_choice: Option<usize>, // The selected choice for the current day
    battle_logs: Vec<BattleLog>,
    menu_select_state: TableState,
}

#[derive(Debug, Default)]
struct BattleLog {
    success: bool,
    title: String,
    day: usize,
    progress: usize,
    choice: Choice,
}

impl GamePage {
    pub fn new() -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        GamePage {
            name: NAME.to_string(),
            action_sender: None,
            deck: None,
            days: 1,
            progress: 0,
            today_card: None,
            should_go_next_day: true,
            finished_encounter_cards: Vec::new(),
            popup: NoPopUp,
            selected_choice: None,
            battle_logs: Vec::new(),
            menu_select_state: state,
        }
    }
}

impl Component for GamePage {
    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match self.popup {
            NoPopUp => {
                let mut idx = self.menu_select_state.selected().unwrap();
                if key.code == KeyCode::Char('1') {
                    self.popup = if self.today_card.is_some() { ChallengeSuccessfulPopup } else { NoPopUp };
                    self.selected_choice = Some(0);
                    info!("[{}] Selected choice 1", self.name);
                }
                if key.code == KeyCode::Char('2') {
                    self.popup = if self.today_card.is_some() { ChallengeSuccessfulPopup } else { NoPopUp };
                    self.selected_choice = Some(1);
                    info!("[{}] Selected choice 2", self.name);
                }
                if key.code == KeyCode::Char('3') {
                    self.popup = if self.today_card.is_some() { ChallengeSuccessfulPopup } else { NoPopUp };
                    self.selected_choice = Some(2);
                    info!("[{}] Selected choice 3", self.name);
                }
                if key.code == KeyCode::Up {
                    if idx > 0 {
                        idx -= 1;
                    }
                    self.menu_select_state.select(Some(idx));
                }
                if key.code == KeyCode::Down {
                    if idx < self.battle_logs.len() - 1 {
                        idx += 1;
                    }
                    self.menu_select_state.select(Some(idx));
                }
            }
            ChallengeSuccessfulPopup => {
                if key.code == KeyCode::Char('y') {
                    self.popup = BreakPopup(None);
                    let today_card = self.today_card.as_ref().unwrap();
                    let today_progress = today_card.progress[self.selected_choice.unwrap()];
                    self.progress += today_progress;
                    info!("[{}] Choice {} challenge successful", self.name, self.selected_choice.unwrap() + 1);
                    self.battle_logs.push(BattleLog {
                        success: true,
                        title: today_card.title.clone(),
                        day: self.days,
                        progress: today_progress,
                        choice: today_card.choices[self.selected_choice.unwrap()].clone(),
                    })
                }
                if key.code == KeyCode::Char('n') {
                    self.popup = BreakPopup(None);
                    let today_card = self.today_card.as_ref().unwrap();
                    info!("[{}] Choice {} challenge failed", self.name, self.selected_choice.unwrap());
                    let mut log = BattleLog::default();
                    log.title = today_card.title.clone();
                    log.day = self.days;
                    log.choice = today_card.choices[self.selected_choice.unwrap()].clone();
                    self.battle_logs.push(log);
                }
                if key.code == KeyCode::Char('p') {
                    self.popup = NoPopUp;
                    self.selected_choice = None;
                    info!("[{}] Went back", self.name);
                }
            }
            BreakPopup(_) => {
                let mut next_day = false;
                if key.code == KeyCode::Char('a') {
                    next_day = true;
                }
                if key.code == KeyCode::Char('b') {
                    next_day = true;
                    self.deck.as_mut().unwrap().shuffle(PutCurrentCardTop, self.today_card.take());
                }
                if key.code == KeyCode::Char('c') {
                    next_day = true;
                    self.deck.as_mut().unwrap().shuffle(FirstTyrantCardTopAndShuffleRest, None);
                }
                if key.code == KeyCode::Char('d') {
                    next_day = true;
                    self.deck.as_mut().unwrap().shuffle(PutCurrentCardRandom, self.today_card.take());
                }
                if key.code == KeyCode::Char('e') {
                    self.days -= 1;
                    next_day = true;
                    self.deck.as_mut().unwrap().shuffle(ReplaceTodayEncounterAndShuffleTodayEncounter, self.today_card.take());
                }
                if key.code == KeyCode::Char('o') {
                    info!("[{}] Selected boss challenge", self.name);
                    let min_required = self.deck.as_ref().unwrap().tyrant_card.min_progress;
                    if self.progress < min_required {
                        self.popup = BreakPopup(Some(format!("无法挑战，最小进度要求: {}, 当前：{}", min_required, self.progress)));
                    } else {
                        self.popup = NoPopUp;
                        self.deck.as_mut().unwrap().encounter_cards.clear();
                        self.days += 1;
                    }
                }
                let mut special_encounters = SPECIAL_ENCOUNTER_CARDS.lock().unwrap();
                for (i, _) in special_encounters.iter().enumerate() {
                    let code_point = 'f' as u32;
                    let new_code_point = code_point + i as u32;
                    let new_char = std::char::from_u32(new_code_point).unwrap();
                    if key.code == KeyCode::Char(new_char) {
                        next_day = true;
                        self.deck.as_mut().unwrap().shuffle(PickSpecialCardAndShuffle, Some(special_encounters.remove(i)));
                        break;
                    }
                }
                if next_day {
                    self.popup = NoPopUp;
                    self.should_go_next_day = true;
                    self.days += 1;
                }
            }
        }
        Ok(())
    }
    fn update(&mut self, action: Action) -> color_eyre::Result<()> {
        match action {
            Action::Update(_, tyrant_name) => {
                let deck = EncounterDeck::new(tyrant_name.as_str());
                self.deck = Some(deck);
            }
            _ => {}
        }
        Ok(())
    }
    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> color_eyre::Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(18),
                Constraint::Percentage(55),
                Constraint::Percentage(27),
            ])
            .split(area);

        let days = String::from("Days: ") + self.days.to_string().as_str();
        let progress = String::from("Progress: ") + self.progress.to_string().as_str();
        // days banner
        let days_banner = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::new())
            .lines(vec![
                days.as_str().yellow().into(),
                progress.as_str().green().into(),
            ])
            .alignment(Alignment::Center)
            .build()?;

        let table = build_battle_log_menu(&self.battle_logs);

        frame.render_widget(days_banner, layout[0]);
        frame.render_stateful_widget(table, layout[2], &mut self.menu_select_state);

        let deck = self.deck.as_mut().unwrap();
        let content;
        if self.days < deck.tyrant_card.max_days && !deck.encounter_cards.is_empty() {
            if self.should_go_next_day {
                info!("[{}] Went next day.", self.name);
                match self.today_card.take() {
                    None => {}
                    Some(today_card) => { self.finished_encounter_cards.push(today_card) }
                }
                let card = deck.encounter_cards.remove(0);
                self.today_card = Some(card);
                self.should_go_next_day = false;
            }
            match self.today_card.as_ref() {
                None => {
                    let card = self.finished_encounter_cards.last().unwrap();
                    content = build_encounter_content(card, self.days);
                }
                Some(card) => {
                    content = build_encounter_content(card, self.days);
                }
            }
        } else {
            match self.today_card.take() {
                None => {}
                Some(today_card) => { self.finished_encounter_cards.push(today_card) }
            }

            let card = &self.deck.as_ref().unwrap().tyrant_card;
            if self.progress >= card.min_progress {
                let mut intro = "Boss战：".to_string() + card.battle_title.as_str();
                intro = intro + "\n------------------\n";
                intro = intro + card.description.as_str();
                intro = intro + "\n------------------\n战斗机制：\n";
                for m in &card.battle_mechanism {
                    intro = intro + " *" + m.as_str() + "\n";
                }
                intro = intro + "\n------------------\nBoss技能：\n";
                for s in &card.tyrant_skills {
                    intro = intro + " *" + s.as_str() + "\n";
                }
                intro = intro + "\n------------------\nBoss骰子：\n";
                for d in &card.tyrant_die {
                    intro = intro + " *" + d.as_str() + "\n";
                }
                content = intro;
            } else {
                content = format!("游戏结束。进度点没有达到要求 {}", card.min_progress);
            }
        }

        let instruction = Title::from(" <1> 第一个选择 || <2> 第二个选择 || <3> 第三个选择 || <Q> 强制退出 ".bold());
        frame.render_widget(
            Paragraph::new(content)
                .wrap(Wrap { trim: true })
                .block(Block::new().borders(Borders::ALL)
                    .border_set(border::THICK)
                    .title(instruction.alignment(Alignment::Center).position(Position::Top))),
            layout[1]);

        match &self.popup {
            NoPopUp => {}
            ChallengeSuccessfulPopup => {
                let popup_area = centered_rect(area, 30, 30);
                let today_card = self.today_card.as_ref().unwrap();
                let content = format!("是否挑战成功？{}\n\n选择: \n{:?}", today_card.title, today_card.choices[self.selected_choice.unwrap()].description);
                let popup = Popup::new(content, "".to_string(), " <Y> 成功 || <N> 失败 || <P> 键回退 ".to_string());
                frame.render_widget(popup, popup_area);
            }
            BreakPopup(content) => {
                let popup_area = centered_rect(area, 40, 40);
                let mut content = match content {
                    None => { "休息一下......".to_string() }
                    Some(word) => { word.clone() }
                };
                content = content + "\n\n\n<a> 无操作进入下一天。\n<b> 将当前遭遇卡放置牌堆顶部。\n<c> 将卡组中第一个暴君遭遇卡置顶。洗剩余的卡。\n<d> 将当前遭遇卡洗入牌堆。\n<e> 为今天抽取新的遭遇卡，并把当前遭遇卡洗入牌堆。\n";
                for (i, card) in SPECIAL_ENCOUNTER_CARDS.lock().unwrap().iter().enumerate() {
                    let code_point = 'f' as u32;
                    let new_code_point = code_point + i as u32;
                    let new_char = std::char::from_u32(new_code_point).unwrap();
                    content += format!("<{}> 将特殊遭遇卡-“{}”洗入牌堆。\n", new_char, card.title).as_str()
                }
                let popup = Popup::new(content, "".to_string(), " <O> 键挑战Boss ".to_string());
                frame.render_widget(popup, popup_area);
            }
        }

        Ok(())
    }
}

fn build_encounter_content(card: &EncounterCard, days: usize) -> String {
    let mut content = String::new();
    content = content + format!("第{}天：", days).as_str() + card.title.as_str() + "\n\n-------------------------------------\n\n";
    content = content + card.story.as_str() + "\n\n-------------------------------------\n\n";
    content = content + "选择：\n\n";
    for c in &card.choices {
        content = content + "*" + c.description.as_str() + "\n";
        content = content + "行动：" + c.action.as_str() + "\n";
        content = content + "奖励：" + c.rewards.as_str() + "\n\n";
    }
    content = content + card.remark.as_str() + "\n\n";
    content = content + "进度：\n";
    for (idx, p) in card.progress.iter().enumerate() {
        content = content + format!("* 选择{}：获取{}进度。\n", idx + 1, p).as_str();
    }
    content
}

fn build_battle_log_menu(battle_log: &Vec<BattleLog>) -> Table {
    let mut rows: Vec<Row> = Vec::new();
    for log in battle_log {
        let r = Row::new(vec![log.day.to_string(),
                              log.title.clone(),
                              log.choice.description.clone(),
                              log.progress.to_string(),
                              if log.success { log.choice.rewards.clone() } else { "无奖励".to_string() },
                              if log.success { "✅".to_string() } else { "❌".to_string() }]).height(2);
        rows.push(r);
    }
    let widths = [
        Constraint::Length(10),
        Constraint::Length(20),
        Constraint::Max(100),
        Constraint::Length(10),
        Constraint::Max(100),
        Constraint::Length(10)
    ];

    let block = Block::default()
        .title(Title::from(" <Enter> 键回滚".bold()).alignment(Alignment::Center).position(Position::Bottom))
        .borders(Borders::ALL)
        .padding(Padding::top(1))
        .border_set(border::THICK);

    let table = Table::new(rows, widths)
        .column_spacing(1)
        .style(Style::new().white())
        .header(
            Row::new(vec!["天数", "战斗", "选择", "进度", "奖励", "是否成功"])
                .style(Style::new().bold())
                .bottom_margin(1),
        )
        .block(block)
        .highlight_style(Style::new().yellow())
        .highlight_symbol(" >> ");
    table
}

enum ShowPopup {
    NoPopUp,
    ChallengeSuccessfulPopup,
    BreakPopup(Option<String>),
}