use crossterm::event::KeyEvent;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;
use color_eyre::Result;
use tui_big_text::{BigText, PixelSize};
use crate::app::{Action};
use crate::component::Component;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, List, ListDirection, ListItem, ListState, Padding};
use ratatui::widgets::block::{Position, Title};
use tracing::info;
use crate::app::Action::{Quit, Render};
use crate::components::{select_boss_page};
use crate::utils::{centered_rect};

/// Home page for the game. Main menu. Menu items:
/// - 开始游戏: Start the game. Emit `Render("SelectBossPage")` event to let APP render the next page.
/// - 退出: exits the application.
pub const NAME: &str = "HomePage";

pub struct HomePage {
    pub name: String,
    pub action_sender: Option<UnboundedSender<Action>>,
    menu_select_state: ListState,
}

impl HomePage {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        HomePage {
            name: NAME.to_string(),
            action_sender: None,
            menu_select_state: state,
        }
    }
}

impl Component for HomePage {
    fn register_action_handler(&mut self, sender: UnboundedSender<Action>) -> Result<()> {
        self.action_sender = Some(sender);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<()> {
        let mut idx = self.menu_select_state.selected().unwrap();
        if key.code == KeyCode::Up {
            if idx > 0 {
                idx -= 1;
            }
            self.menu_select_state.select(Some(idx));
        }
        if key.code == KeyCode::Down {
            if idx < 3 {
                idx += 1;
            }
            self.menu_select_state.select(Some(idx));
        }
        if key.code == KeyCode::Enter {
            match idx {
                0 => {
                    info!("[{}] Selected 开始游戏", self.name);
                    self.action_sender.as_mut().unwrap().send(Render(select_boss_page::NAME.to_string()))?;
                }
                1 => {
                    info!("[{}] Selected 退出", self.name);
                    self.action_sender.as_mut().unwrap().send(Quit)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Banner
        let tmb_banner = BigText::builder()
            .pixel_size(PixelSize::Full)
            .style(Style::new())
            .lines(vec![
                "Too Many Bones".red().into(),
            ])
            .alignment(Alignment::Center)
            .build()?;
        // Border
        let instruction = Title::from(" <Enter> 键选择 || <Q> 键强制退出".bold());
        let block = Block::default()
            .title(instruction.alignment(Alignment::Center).position(Position::Bottom))
            .borders(Borders::ALL)
            .padding(Padding::top(1))
            .border_set(border::THICK);
        // Menu
        let menu_items: Vec<ListItem> = vec![ListItem::new("开始新游戏"), ListItem::new("退出")];

        let list = List::new(menu_items)
            .block(Block::bordered().title(Title::from(" 主菜单 ".bold()).alignment(Alignment::Center)))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().yellow().bold())
            .highlight_symbol(" ☠️ ")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);


        let inner_area = block.inner(f.size());

        // Split the area, top is banner, bottom is menu
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(inner_area);

        f.render_widget(block, area);
        f.render_widget(tmb_banner, layout[0]);
        f.render_stateful_widget(list, centered_rect(layout[1], 50, 50), &mut self.menu_select_state);
        Ok(())
    }
}