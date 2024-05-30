use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Constraint, Style, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Padding, Row, Table, TableState};
use ratatui::widgets::block::{Position, Title};
use tokio::sync::mpsc::UnboundedSender;
use crate::app::Action;
use crate::app::Action::Render;
use crate::component::Component;
use crate::components::{home_page};
use crate::core::game_info::TyrantCard;

pub const NAME: &str = "SelectBossPage";

pub struct SelectBossPage {
    pub name: String,
    pub action_sender: Option<UnboundedSender<Action>>,
    pub menu_select_state: TableState,
}

impl SelectBossPage {
    pub fn new() -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        SelectBossPage {
            name: NAME.to_string(),
            action_sender: None,
            menu_select_state: state,
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
            self.action_sender.as_mut().unwrap().send(Render(home_page::NAME.to_string()))?;
        }
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> color_eyre::Result<()> {
        let tyrants = TyrantCard::list();
        let mut rows: Vec<Row> = Vec::new();
        for card in tyrants {
            let mut intro = card.battle_title;
            intro = intro + "\n-----------\n战斗机制：\n";
            for m in card.battle_mechanism {
                intro = intro + m.as_str();
            }
            intro = intro + "\n-----------\nBoss技能：\n";
            for s in card.tyrant_skills {
                intro = intro + s.as_str();
            }
            intro = intro + "\n-----------\nBoss骰子：\n";
            for d in card.tyrant_die {
                intro = intro + d.as_str();
            }

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
            .style(Style::new().blue())
            .header(
                Row::new(vec!["Boss", "简介", "游戏时长", "最小挑战天数", "最大挑战天数", "怪物类型"])
                    .style(Style::new().bold())
                    .bottom_margin(1),
            )
            .block(block)
            .highlight_style(Style::new().reversed())
            .highlight_symbol(" >> ");

        f.render_stateful_widget(table, area, &mut self.menu_select_state);
        Ok(())
    }
}