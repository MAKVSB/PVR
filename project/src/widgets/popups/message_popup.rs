use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

use super::popup::PopupEvent;

#[derive(Debug)]
pub struct MessagePopup {
    pub message: String,
    pub title: String,
}
impl MessagePopup {
    pub fn new(title: String, message: String) -> Self {
        Self { title, message }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        /// helper function to create a centered rect using up certain percentage of the available rect `r`
        fn popup_area(area: Rect, percent_x: u16, size_y: u16) -> Rect {
            let vertical = Layout::vertical([Constraint::Length(size_y)]).flex(Flex::Center);
            let horizontal =
                Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
            let [area] = vertical.areas(area);
            let [area] = horizontal.areas(area);
            area
        }

        let block = Paragraph::new(self.message.as_str())
            .block(Block::bordered().title(self.title.as_str()));
        let rows_num = self.message.split("\n").collect::<Vec<&str>>().len() as u16;
        let area = popup_area(area, 60, 2 + rows_num);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(block, area);
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> PopupEvent {
        match key_event.code {
            KeyCode::Esc | KeyCode::Enter => PopupEvent::PopupClose,
            _ => PopupEvent::None,
        }
    }
}
