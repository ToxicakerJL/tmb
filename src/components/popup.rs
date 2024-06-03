use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Style, Widget};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};
use ratatui::widgets::block::{Position, Title};

#[derive(Default)]
pub struct Popup {
    content: String,
    title: String,
    command: String,
}

impl Popup {
    pub fn new(content: String, title: String, command: String) -> Self {
        Popup {
            content,
            title,
            command,
        }
    }
}

impl Widget for Popup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let larger_area = Rect::new(area.x - 1, area.y - 1, area.width + 1, area.height + 1);
        Clear.render(larger_area, buf);
        let popup_block = Block::default()
            .title(Title::from(self.title.as_str()).alignment(Alignment::Center).position(Position::Top))
            .borders(Borders::ALL)
            .padding(Padding::new(1, 1, 1, 1))
            .title(Title::from(self.command.as_str()).alignment(Alignment::Center).position(Position::Bottom))
            .style(Style::default().bg(Color::DarkGray));
        Paragraph::new(self.content.as_str())
            .wrap(Wrap { trim: true })
            .block(popup_block)
            .render(area, buf);
    }
}