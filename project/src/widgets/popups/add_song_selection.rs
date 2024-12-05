use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::Clear,
    Frame,
};

use crate::{
    types::music_types::{PlaylistIdWrapper, RSyncSong},
    widgets::generic::list_selector::{ListSelector, ListSelectorKeyResponse, ListSelectorLabels},
};

use super::popup::PopupEvent;

#[derive(Debug)]
pub struct AddSongSelectionPopup {
    pub playlist_id: PlaylistIdWrapper,
    pub selector: ListSelector<RSyncSong>,
}
impl<'a> AddSongSelectionPopup {
    pub fn new(items: Vec<RSyncSong>, playlist_id: PlaylistIdWrapper) -> Self {
        Self {
            playlist_id,
            selector: ListSelector::new(
                Some(items),
                ListSelectorLabels {
                    empty: "".into(),
                    title: "Select song to add".into(),
                },
                false,
            ),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        /// helper function to create a centered rect using up certain percentage of the available rect `r`
        fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
            let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
            let horizontal =
                Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
            let [area] = vertical.areas(area);
            let [area] = horizontal.areas(area);
            area
        }

        let area = popup_area(area, 60, 20);
        frame.render_widget(Clear, area); //this clears out the background
                                          // Create a List from all list items and highlight the currently selected one

        self.selector.render(frame, area, true);
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> PopupEvent {
        match self.selector.handle_key_events(key_event) {
            ListSelectorKeyResponse::Selected => {
                if let Some(item) = self.selector.get_selected_items().first() {
                    PopupEvent::PopupCloseData(item.id.clone())
                } else {
                    PopupEvent::None
                }
            }
            ListSelectorKeyResponse::CursorMoved => PopupEvent::None,
            ListSelectorKeyResponse::None => PopupEvent::None,
            ListSelectorKeyResponse::Pass => PopupEvent::Pass,
        }
    }
}
