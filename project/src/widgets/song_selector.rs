use crossterm::event::{KeyCode, KeyEvent, };
use ratatui::{
    layout::Rect, widgets::ListState, Frame
};

use crate::types::{music_types::RSyncSong, playlist_selector_key_event_response::SelectorKeyEventResponse};

use super::generic::list_selector::{ListSelector, ListSelectorKeyResponse, ListSelectorLabels};

#[derive(Debug)]
pub struct SongSelector {
    pub active: bool,
    pub selector: ListSelector<RSyncSong>
}
impl SongSelector {
    pub fn new(title: String) -> Self {
        let mut st: ListState = ListState::default();
        st.select(Some(0));
        Self {
            active: false,
            selector: ListSelector::new(None, ListSelectorLabels {
                empty: "Waiting for playlist to be selected".into(),
                title,
            }, true)
        }
    }
    
    pub fn get_selected(&mut self) -> Vec<&RSyncSong>{
        self.selector.get_selected_items()
    }

    pub fn clear_selected(&mut self) {
        self.selector.clear_selected();
    }

    pub fn set_items(&mut self, items: Option<Vec<RSyncSong>>) {
        self.selector.set_items(items);
    }

    pub fn set_loading(&mut self) {
        self.selector.set_loading();
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.selector.render(frame, area, self.active);
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> SelectorKeyEventResponse {
        match self.selector.handle_key_events(key_event) {
            ListSelectorKeyResponse::Selected => return SelectorKeyEventResponse::None,
            ListSelectorKeyResponse::CursorMoved => return SelectorKeyEventResponse::None,
            ListSelectorKeyResponse::None => return SelectorKeyEventResponse::None,
            ListSelectorKeyResponse::Pass => (),
        };

        match key_event.code {
            KeyCode::Char('r') => {
                SelectorKeyEventResponse::Refresh
            }
            KeyCode::Char('o') => {
                if let Some(item) = self.selector.get_cursor_item() {
                    let _ = webbrowser::open(item.url.as_str());
                }
                SelectorKeyEventResponse::None
            }
            _ => SelectorKeyEventResponse::Pass
        }
    }
}