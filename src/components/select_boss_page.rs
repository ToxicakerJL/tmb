use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Row, Table, TableState, Wrap};
use ratatui::widgets::block::{Position, Title};
use tokio::sync::mpsc::UnboundedSender;
use crate::app::Action;
use crate::app::Action::Render;
use crate::component::Component;
use crate::components::{home_page};
use crate::core::game_info::TyrantCard;
use crate::utils::centered_rect;

pub const NAME: &str = "SelectBossPage";

pub struct SelectBossPage {
    pub name: String,
    pub action_sender: Option<UnboundedSender<Action>>,
    pub menu_select_state: TableState,
    pub is_popup: bool,
}

#[derive(Default)]
struct BossInfoPopup {
    content: String,
}

impl Widget for BossInfoPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let popup_block = Block::default()
            .title(Title::from(" Boss介绍 ").alignment(Alignment::Center).position(Position::Top))
            .borders(Borders::ALL)
            .padding(Padding::new(1, 1, 1, 1))
            .title(Title::from(" <P> 键回退 || <Enter> 键选择Boss进行游戏 ").alignment(Alignment::Center).position(Position::Bottom))
            .style(Style::default().bg(Color::DarkGray));
        Paragraph::new(self.content)
            .wrap(Wrap { trim: true })
            .block(popup_block)
            .render(area, buf);
    }
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
            if !self.is_popup {
                self.action_sender.as_mut().unwrap().send(Render(home_page::NAME.to_string()))?;
            } else {
                self.is_popup = false;
            }
        }
        if key.code == KeyCode::Enter {
            if !self.is_popup {
                self.is_popup = true;
            }
            if self.is_popup {
                //Todo next page
            }
        }
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> color_eyre::Result<()> {
        let tyrants = TyrantCard::list();
        let mut rows: Vec<Row> = Vec::new();
        let mut boss_intro_list = Vec::new();
        for card in tyrants {
            let mut intro = card.battle_title;
            intro = intro + "\n------------------\n战斗机制：\n";
            for m in card.battle_mechanism {
                intro = intro + " *" + m.as_str() + "\n";
            }
            intro = intro + "\n------------------\nBoss技能：\n";
            for s in card.tyrant_skills {
                intro = intro + " *" + s.as_str() + "\n";
            }
            intro = intro + "\n------------------\nBoss骰子：\n";
            for d in card.tyrant_die {
                intro = intro + " *" + d.as_str() + "\n";
            }

            boss_intro_list.push(intro);

            let desc = card.description.replace("。", "。\n");

            let r = Row::new(vec![card.name,
                                  desc,
                                  card.game_length,
                                  card.min_days.to_string(),
                                  card.max_days.to_string(),
                                  card.creatures]).height(6);
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
            let mut popup = BossInfoPopup::default();
            let idx = self.menu_select_state.selected().unwrap();
            popup.content = boss_intro_list[idx].clone();
            f.render_widget(popup, popup_area);
        }
        Ok(())
    }
}