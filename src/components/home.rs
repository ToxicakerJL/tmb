use crossterm::event::KeyEvent;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;
use color_eyre::Result;
use tui_big_text::{BigText, PixelSize};
use crate::app::{Action};
use crate::app::Action::Render;
use crate::component::Component;
use crate::components::next;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders};
use ratatui::widgets::block::{Position, Title};


pub const NAME: &str = "Home";

pub struct Home {
    pub name: String,
    pub action_sender: Option<UnboundedSender<Action>>,
}

impl Home {
    pub fn new() -> Self {
        Home {
            name: NAME.to_string(),
            action_sender: None,
        }
    }
}

impl Component for Home {
    fn register_action_handler(&mut self, sender: UnboundedSender<Action>) -> Result<()> {
        self.action_sender = Some(sender);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Right {
            self.action_sender.as_mut().unwrap().send(Render(next::NAME.to_string()))?;
        }
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let tmb_banner = BigText::builder()
            .pixel_size(PixelSize::Full)
            .style(Style::new())
            .lines(vec![
                "Too Many Bones".red().into(),
            ])
            .build()?;

        let instruction = Title::from(" Type <Q> to exit ".bold());

        let block = Block::default()
            .title(instruction.alignment(Alignment::Center).position(Position::Bottom))
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let inner_area = block.inner(f.size());
        f.render_widget(block, area);
        f.render_widget(tmb_banner, inner_area);
        Ok(())
    }
}