use crossterm::event::KeyEvent;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use tokio::sync::mpsc::UnboundedSender;
use color_eyre::Result;
use crate::app::{Action};
use crate::app::Action::Render;
use crate::component::Component;
use crate::components::next;

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
        f.render_widget(Paragraph::new("hello world"), area);
        Ok(())
    }
}