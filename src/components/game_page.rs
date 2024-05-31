use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use tokio::sync::mpsc::UnboundedSender;
use tui_big_text::{BigText, PixelSize};
use crate::app::Action;
use crate::component::Component;
use crate::core::game::{EncounterDeck, TyrantCard};

pub const NAME: &str = "GamePage";

pub struct GamePage {
    pub name: String,
    pub action_sender: Option<UnboundedSender<Action>>,
    pub deck: Option<EncounterDeck>,
    pub days: usize,
}

impl GamePage {
    pub fn new() -> Self {
        GamePage {
            name: NAME.to_string(),
            action_sender: None,
            deck: None,
            days: 1,
        }
    }
}

impl Component for GamePage {
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
                Constraint::Percentage(25),
                Constraint::Percentage(75),
            ])
            .split(area);

        let days = String::from("Days ") + self.days.to_string().as_str();
        // days banner
        let days_banner = BigText::builder()
            .pixel_size(PixelSize::Full)
            .style(Style::new())
            .lines(vec![
                days.as_str().yellow().into(),
            ])
            .alignment(Alignment::Center)
            .build()?;

        frame.render_widget(days_banner, layout[0]);

        frame.render_widget(
            Paragraph::new("inner 1")
                .block(Block::new().borders(Borders::ALL)),
            layout[1]);
        Ok(())
    }
}