use crossterm::event::{KeyCode, KeyEvent, };
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect}, widgets::Clear, Frame
};

use crate::{providers::provider_traits::APIProvider, types::music_types::PlaylistIdWrapper, widgets::generic::user_input::{UserInput, UserInputKeyEvent}};

use super::{add_song_selection::AddSongSelectionPopup, popup::{Popup, PopupEvent}};


#[derive(Debug)]
pub struct AddSongPopup<T>
where 
    T: APIProvider + Clone
{
    pub provider: T,
    pub user_input: UserInput,
    pub playlist_id: PlaylistIdWrapper,
}
impl<T> AddSongPopup<T> 
where 
    T: APIProvider + Clone
{
    pub fn new(provider: T, playlist_id: PlaylistIdWrapper) -> Self {
        Self {
            provider,
            user_input: UserInput::new(true),
            playlist_id,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        /// helper function to create a centered rect using up certain percentage of the available rect `r`
        fn popup_area(area: Rect, percent_x: u16, _percent_y: u16) -> Rect {
            let vertical = Layout::vertical([Constraint::Length(3)]).flex(Flex::Center);
            let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
            let [area] = vertical.areas(area);
            let [area] = horizontal.areas(area);
            area
        }

        let area = popup_area(area, 60, 20);
        frame.render_widget(Clear, area); //this clears out the background
        self.user_input.render(frame, area);
    }

    pub async fn handle_key_events(&mut self, key_event: KeyEvent) -> PopupEvent<T> {
        match self.user_input.handle_key_events(key_event) {
            UserInputKeyEvent::None => return PopupEvent::None,
            UserInputKeyEvent::Pass => {}, //pass keypress to next parser
            UserInputKeyEvent::Data(song_name) => {
                if !song_name.is_empty() {
                    return PopupEvent::PopupCloseDataPopup(Popup::AddSongSelect(AddSongSelectionPopup::<T>::new(self.provider.clone(), song_name, self.playlist_id.clone()).await))
                } else {
                    return PopupEvent::None
                }
            },
        }

        match key_event.code {
            KeyCode::Esc => PopupEvent::PopupClose,
            _ => PopupEvent::None
        }
    }
}