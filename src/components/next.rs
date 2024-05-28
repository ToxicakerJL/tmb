use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use tokio::sync::mpsc::UnboundedSender;
use crate::app::Action;
use crate::app::Action::Render;
use crate::component::Component;
use crate::components::{home};

pub const NAME: &str = "Next";

pub struct Next {
    pub name: String,
    pub action_sender: Option<UnboundedSender<Action>>,
}

impl Next {
    pub fn new() -> Self {
        Next {
            name: NAME.to_string(),
            action_sender: None,
        }
    }
}

impl Component for Next {
    fn register_action_handler(&mut self, sender: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.action_sender = Some(sender);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        if key.code == KeyCode::Left {
            self.action_sender.as_mut().unwrap().send(Render(home::NAME.to_string()))?;
        }
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> color_eyre::Result<()> {
        f.render_widget(Paragraph::new("hello world123"), area);
        Ok(())
    }
}