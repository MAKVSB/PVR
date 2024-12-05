use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::Clear,
    Frame,
};

use crate::widgets::generic::user_input::{UserInput, UserInputKeyEvent};

use super::popup::PopupEvent;

#[derive(Debug)]
pub struct AddPlaylistPopup {
    pub user_input: UserInput,
}
impl AddPlaylistPopup {
    pub fn new() -> Self {
        Self {
            user_input: UserInput::new(true),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        /// helper function to create a centered rect using up certain percentage of the available rect `r`
        fn popup_area(area: Rect, percent_x: u16, _percent_y: u16) -> Rect {
            let vertical = Layout::vertical([Constraint::Length(3)]).flex(Flex::Center);
            let horizontal =
                Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
            let [area] = vertical.areas(area);
            let [area] = horizontal.areas(area);
            area
        }

        let area = popup_area(area, 60, 20);
        frame.render_widget(Clear, area); //this clears out the background
        self.user_input.render(frame, area);
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> PopupEvent {
        match self.user_input.handle_key_events(key_event) {
            UserInputKeyEvent::None => return PopupEvent::None,
            UserInputKeyEvent::Pass => {} //pass keypress to next parser
            UserInputKeyEvent::Data(playlist_name) => {
                return PopupEvent::PopupCloseData(playlist_name);
            }
        }

        match key_event.code {
            KeyCode::Esc => PopupEvent::PopupClose,
            _ => PopupEvent::None,
        }
    }
}
