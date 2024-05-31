use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListDirection, Padding, Paragraph, Wrap};
use ratatui::widgets::block::{Position, Title};
use tokio::sync::mpsc::UnboundedSender;
use tui_big_text::{BigText, PixelSize};
use crate::app::Action;
use crate::component::Component;
use crate::core::game::{EncounterCard, EncounterDeck};
use crate::core::game::ShuffleStrategy::{FirstTyrantCardTopAndShuffleRest, PickSpecialCardAndShuffle, PutCurrentCardRandom, PutCurrentCardTop};
use crate::utils::centered_rect;

pub const NAME: &str = "GamePage";

pub struct GamePage {
    pub name: String,
    pub action_sender: Option<UnboundedSender<Action>>,
    pub deck: Option<EncounterDeck>,
    pub days: usize,
    pub progress: usize,
    pub today_card: Option<EncounterCard>,
    pub should_go_next_day: bool,
    pub finished_encounter_cards: Vec<(EncounterCard, bool)>,
    pub challenge_successful_popup: (bool, usize),
    pub take_break_popup: (bool, String),
    pub special_encounters: Vec<EncounterCard>,
}

#[derive(Default)]
struct ChallengeSuccessfulPopup {
    content: String,
}

impl Widget for ChallengeSuccessfulPopup {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let larger_area = Rect::new(area.x - 1, area.y - 1, area.width + 1, area.height + 1);
        Clear.render(larger_area, buf);
        let popup_block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::new(1, 1, 1, 1))
            .title(Title::from(" <Y> 成功 || <N> 失败 || <P> 键回退").alignment(Alignment::Center).position(Position::Bottom))
            .style(Style::default().bg(Color::DarkGray));
        Paragraph::new(self.content)
            .wrap(Wrap { trim: true })
            .block(popup_block)
            .render(area, buf);
    }
}

#[derive(Default)]
struct TakeBreakPopup {
    content: String,
}

impl Widget for TakeBreakPopup {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let larger_area = Rect::new(area.x - 1, area.y - 1, area.width + 1, area.height + 1);
        Clear.render(larger_area, buf);

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::new(1, 1, 1, 1))
            .title(Title::from(" 任意选择将会进入下一天 || <O> 挑战Boss").alignment(Alignment::Center).position(Position::Bottom))
            .style(Style::default().bg(Color::DarkGray));

        Paragraph::new(self.content)
            .wrap(Wrap { trim: true })
            .block(popup_block)
            .render(area, buf);
    }
}

impl GamePage {
    pub fn new() -> Self {
        GamePage {
            name: NAME.to_string(),
            action_sender: None,
            deck: None,
            days: 1,
            progress: 0,
            today_card: None,
            should_go_next_day: true,
            finished_encounter_cards: Vec::new(),
            challenge_successful_popup: (false, 0),
            take_break_popup: (false, "休息整顿...".to_string()),
            special_encounters: EncounterCard::list_special_cards(),
        }
    }
}

impl Component for GamePage {
    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        let choice_num = match self.today_card.as_ref() {
            None => { 0 }
            Some(card) => { card.choices.len() }
        };
        if key.code == KeyCode::Char('1') && !self.take_break_popup.0 && 1 <= choice_num {
            self.challenge_successful_popup = (true, 0)
        }
        if key.code == KeyCode::Char('2') && !self.take_break_popup.0 && 2 <= choice_num {
            self.challenge_successful_popup = (true, 1)
        }
        if key.code == KeyCode::Char('3') && !self.take_break_popup.0 && 3 <= choice_num {
            self.challenge_successful_popup = (true, 2)
        }
        if key.code == KeyCode::Char('p') && !self.take_break_popup.0 && self.challenge_successful_popup.0 {
            self.challenge_successful_popup = (false, 0)
        }
        if key.code == KeyCode::Char('y') && self.challenge_successful_popup.0 {
            self.challenge_successful_popup = (false, self.challenge_successful_popup.1);
            self.take_break_popup = (true, "休息整顿...".to_string());
            let today_card = self.today_card.clone().unwrap();
            self.progress += today_card.progress[self.challenge_successful_popup.1];
            self.finished_encounter_cards.push((today_card, true));
        }
        if key.code == KeyCode::Char('n') && self.challenge_successful_popup.0 {
            self.challenge_successful_popup = (false, self.challenge_successful_popup.1);
            self.take_break_popup = (true, "休息整顿...".to_string());
            self.finished_encounter_cards.push((self.today_card.clone().unwrap(), false));
        }
        if self.take_break_popup.0 {
            let mut next_day = false;
            if key.code == KeyCode::Char('a') {
                next_day = true;
            }
            if key.code == KeyCode::Char('b') {
                next_day = true;
                self.deck.as_mut().unwrap().shuffle(PutCurrentCardTop, self.today_card.clone());
            }
            if key.code == KeyCode::Char('c') {
                next_day = true;
                self.deck.as_mut().unwrap().shuffle(FirstTyrantCardTopAndShuffleRest, None);
            }
            if key.code == KeyCode::Char('d') {
                next_day = true;
                self.deck.as_mut().unwrap().shuffle(PutCurrentCardRandom, self.today_card.clone());
            }
            for (i, card) in self.special_encounters.iter().enumerate() {
                let code_point = 'e' as u32;
                let new_code_point = code_point + i as u32;
                let new_char = std::char::from_u32(new_code_point).unwrap();
                if key.code == KeyCode::Char(new_char) {
                    next_day = true;
                    self.deck.as_mut().unwrap().shuffle(PickSpecialCardAndShuffle, Some(card.clone()));
                }
            }
            if next_day {
                self.take_break_popup.0 = false;
                self.should_go_next_day = true;
                self.days += 1;
            }
        }
        if key.code == KeyCode::Char('o') && self.take_break_popup.0 {
            let min_required = self.deck.as_ref().unwrap().tyrant_card.min_progress;
            if self.progress < min_required {
                self.take_break_popup.1 = format!("无法挑战，最小进度要求: {}, 当前：{}", min_required, self.progress)
            } else {
                self.take_break_popup = (false, "休息整顿...".to_string());
                self.deck.as_mut().unwrap().encounter_cards.clear();
                self.days += 1;
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
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ])
            .split(area);

        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(layout[0]);

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

        let mut items = Vec::new();
        let mut i = 1;
        for (card, success) in &self.finished_encounter_cards {
            if *success {
                let day = format!("✅ 第{}天：{}", i, card.title.clone());
                items.push(day);
            } else {
                let day = format!("❌ 第{}天：{}", i, card.title.clone());
                items.push(day);
            }
            i += 1;
        }
        let list = List::new(items)
            .direction(ListDirection::TopToBottom);

        frame.render_widget(days_banner, left_layout[0]);
        frame.render_widget(list, centered_rect(left_layout[1], 50, 50));

        let deck = self.deck.as_mut().unwrap();
        let mut content = String::new();
        if self.days < deck.tyrant_card.max_days && !deck.encounter_cards.is_empty() {
            if self.should_go_next_day {
                let card = deck.encounter_cards.remove(0);
                self.today_card = Some(card);
                self.should_go_next_day = false;
            }
            match self.today_card.as_ref() {
                None => { content += "......"; }
                Some(card) => {
                    content = content + format!("第{}天：", self.days).as_str() + card.title.as_str() + "\n\n-------------------------------------\n\n";
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
                }
            }
        } else {
            let card = &self.deck.as_ref().unwrap().tyrant_card;
            let mut intro = card.battle_title.clone();
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
        }

        let instruction = Title::from(" <1> 第一个选择 || <2> 第二个选择 || <3> 第三个选择 || <Q> 强制退出 ".bold());
        frame.render_widget(
            Paragraph::new(content)
                .wrap(Wrap { trim: true })
                .block(Block::new().borders(Borders::ALL).title(instruction.alignment(Alignment::Center).position(Position::Bottom))),
            layout[1]);

        if self.challenge_successful_popup.0 {
            let popup_area = centered_rect(area, 30, 30);
            let mut popup = ChallengeSuccessfulPopup::default();
            let today_card = self.today_card.as_ref().unwrap();
            popup.content = format!("是否挑战成功？{}\n\n选择: \n{:?}", today_card.title, today_card.choices[self.challenge_successful_popup.1].description);
            frame.render_widget(popup, popup_area);
        }

        if self.take_break_popup.0 {
            let popup_area = centered_rect(area, 40, 40);
            let mut popup = TakeBreakPopup::default();
            popup.content = self.take_break_popup.1.clone();
            popup.content = popup.content + "\n\n\n<a> 无操作进入下一天。\n<b> 将当前遭遇卡放置牌堆顶部。\n<c> 找到卡组中的第一个暴君遭遇卡并将其置于顶端。将剩余的卡洗牌并放在下面。\n<d> 将当前遭遇卡洗入牌堆。";
            for (i, card) in self.special_encounters.iter().enumerate() {
                let code_point = 'e' as u32;
                let new_code_point = code_point + i as u32;
                let new_char = std::char::from_u32(new_code_point).unwrap();
                popup.content += format!("<{}> 将特殊遭遇卡-“{}”洗入牌堆。\n", new_char, card.title).as_str()
            }
            frame.render_widget(popup, popup_area);
        }
        Ok(())
    }
}