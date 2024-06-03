use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Padding, Row, Table, TableState};
use ratatui::widgets::block::{Position, Title};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
use crate::app::Action;
use crate::app::Action::{Render, Update};
use crate::component::Component;
use crate::components::{game_page, home_page};
use crate::components::popup::Popup;
use crate::core::game::{TYRANT_CARDS};
use crate::utils::centered_rect;

pub const NAME: &str = "SelectBossPage";

pub struct SelectBossPage {
    pub name: String,
    pub action_sender: Option<UnboundedSender<Action>>,
    menu_select_state: TableState,
    is_popup: bool,
}

impl SelectBossPage {
    pub fn new() -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        SelectBossPage {
            name: NAME.to_string(),
            action_sender: None,
            menu_select_state: state,
            is_popup: false,
        }
    }
}

impl Component for SelectBossPage {
    fn register_action_handler(&mut self, sender: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.action_sender = Some(sender);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        let mut idx = self.menu_select_state.selected().unwrap();
        let tyrant_cards = TYRANT_CARDS.lock().unwrap();
        let tyrant_card = tyrant_cards.get(idx).unwrap();

        match self.is_popup {
            true => {
                if key.code == KeyCode::Char('p') {
                    self.is_popup = false;
                }
                if key.code == KeyCode::Enter {
                    self.is_popup = false;
                    let update = Update(game_page::NAME.to_string(), tyrant_card.id.clone());
                    let render = Render(game_page::NAME.to_string());
                    info!("[{}] Sending action: {:?}, {:?}", self.name, update, render);
                    self.action_sender.as_mut().unwrap().send(update)?;
                    self.action_sender.as_mut().unwrap().send(render)?;
                }
            }
            false => {
                if key.code == KeyCode::Up {
                    if idx > 0 {
                        idx -= 1;
                    }
                    self.menu_select_state.select(Some(idx));
                }
                if key.code == KeyCode::Down {
                    if idx < 6 {
                        idx += 1;
                    }
                    self.menu_select_state.select(Some(idx));
                }
                if key.code == KeyCode::Char('p') {
                    self.action_sender.as_mut().unwrap().send(Render(home_page::NAME.to_string()))?;
                }
                if key.code == KeyCode::Enter {
                    info!("[{}] Checking tyrant card info: {}", self.name, tyrant_card.name);
                    self.is_popup = true;
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> color_eyre::Result<()> {
        let mut rows: Vec<Row> = Vec::new();
        let mut boss_intro_list = Vec::new();
        for card in TYRANT_CARDS.lock().unwrap().iter() {
            let mut intro = card.battle_title.clone();
            intro = intro + "\n------------------\n战斗机制：\n";
            for m in card.battle_mechanism.iter() {
                intro = intro + " *" + m.as_str() + "\n";
            }
            intro = intro + "\n------------------\nBoss技能：\n";
            for s in card.tyrant_skills.iter() {
                intro = intro + " *" + s.as_str() + "\n";
            }
            intro = intro + "\n------------------\nBoss骰子：\n";
            for d in card.tyrant_die.iter() {
                intro = intro + " *" + d.as_str() + "\n";
            }

            boss_intro_list.push(intro);

            let desc = card.description.replace("。", "。\n");

            let r = Row::new(vec![card.name.clone(),
                                  desc,
                                  card.game_length.clone(),
                                  card.min_progress.to_string(),
                                  card.max_days.to_string(),
                                  card.creatures.clone()]).height(6);
            rows.push(r);
        }

        let widths = [
            Constraint::Length(15),
            Constraint::Max(100),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(40)
        ];
        let block = Block::default()
            .title(Title::from(" 选择Boss ").alignment(Alignment::Center).position(Position::Top))
            .title(Title::from(" <P> 键回退上一页 || <Enter> 键选择 || <Q> 键强制退出").alignment(Alignment::Center).position(Position::Bottom))
            .borders(Borders::ALL)
            .padding(Padding::top(1))
            .border_set(border::THICK);

        let table = Table::new(rows, widths)
            .column_spacing(1)
            .style(Style::new().white())
            .header(
                Row::new(vec!["Boss", "简介", "游戏时长", "最小挑战天数", "最大挑战天数", "怪物类型"])
                    .style(Style::new().bold())
                    .bottom_margin(1),
            )
            .block(block)
            .highlight_style(Style::new().yellow())
            .highlight_symbol(" >> ");

        f.render_stateful_widget(table, area, &mut self.menu_select_state);

        if self.is_popup {
            let popup_area = centered_rect(area, 60, 80);
            let idx = self.menu_select_state.selected().unwrap();
            let boss_info_popup = Popup::new(boss_intro_list[idx].clone(), " Boss介绍 ".to_string(), " <P> 键回退 || <Enter> 键选择Boss进行游戏 ".to_string());
            f.render_widget(boss_info_popup, popup_area);
        }

        Ok(())
    }
}