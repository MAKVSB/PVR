use crossterm::event::{KeyCode, KeyEvent, };
use ratatui::{
    layout::Rect, Frame
};

use crate::types::{music_types::RSyncPlaylistItem, playlist_selector_key_event_response::SelectorKeyEventResponse};

use super::generic::list_selector::{ListSelector, ListSelectorKeyResponse, ListSelectorLabels};

#[derive(Debug)]
pub struct PlaylistSelector {
    pub active: bool,
    pub selector: ListSelector<RSyncPlaylistItem>,
}
impl<'a> PlaylistSelector {
    pub fn new(title: String) -> Self {
        Self {
            active: false,
            selector: ListSelector::new(None, ListSelectorLabels {
                empty: "Waiting for playlist to be selected".into(),
                title,
            }, false),
        }
    }

    pub fn get_selected_songs(&mut self) -> Option<Vec<&RSyncPlaylistItem>>{
        self.selector.get_selected_items()
    }

    pub fn clear_selected(&mut self) {
        self.selector.clear_selected();
    }

    pub fn set_items(&mut self, items: Option<Vec<RSyncPlaylistItem>>) {
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
            ListSelectorKeyResponse::Selected => {
                if let Some(playlists) = self.selector.get_selected_items() {
                    if let Some(playlist) = playlists.first() {
                        return SelectorKeyEventResponse::Selected(playlist.id.clone())
                    }
                };
                return SelectorKeyEventResponse::None
            },
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