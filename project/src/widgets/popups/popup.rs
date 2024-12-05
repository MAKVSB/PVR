use crossterm::event::KeyEvent;

use crate::widgets::popups::{
    add_playlist::AddPlaylistPopup, add_song::AddSongPopup,
    add_song_selection::AddSongSelectionPopup, message_popup::MessagePopup,
};

#[derive(Debug)]
pub enum PopupEvent {
    PopupClose,
    PopupCloseRefresh,
    PopupCloseData(String),
    None,
    Pass,
}

#[derive(Debug)]
pub enum GenericPopup {
    Message(MessagePopup),
}

impl GenericPopup {
    pub fn render(&mut self, frame: &mut ratatui::Frame<'_>, area: ratatui::prelude::Rect) {
        match self {
            GenericPopup::Message(message_popup) => message_popup.render(frame, area),
        }
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> PopupEvent {
        match self {
            GenericPopup::Message(message_popup) => message_popup.handle_key_events(key_event),
        }
    }
}

#[derive(Debug)]
pub enum PlatformPopup {
    AddSong(AddSongPopup),
    AddSongSelect(AddSongSelectionPopup),
    AddPlaylist(AddPlaylistPopup),
}

impl PlatformPopup {
    pub fn render(&mut self, frame: &mut ratatui::Frame<'_>, area: ratatui::prelude::Rect) {
        match self {
            PlatformPopup::AddSong(popup) => popup.render(frame, area),
            PlatformPopup::AddSongSelect(popup) => popup.render(frame, area),
            PlatformPopup::AddPlaylist(popup) => popup.render(frame, area),
        }
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> PopupEvent {
        match self {
            PlatformPopup::AddSong(popup) => popup.handle_key_events(key_event),
            PlatformPopup::AddSongSelect(popup) => popup.handle_key_events(key_event),
            PlatformPopup::AddPlaylist(popup) => popup.handle_key_events(key_event),
        }
    }
}

#[derive(Debug)]
pub enum PopupTyped {
    Spotify(PlatformPopup),
    Youtube(PlatformPopup),
    Generic(GenericPopup),
}
impl PopupTyped {
    pub fn render(&mut self, frame: &mut ratatui::Frame<'_>, area: ratatui::prelude::Rect) {
        match self {
            PopupTyped::Youtube(popup) | PopupTyped::Spotify(popup) => popup.render(frame, area),
            PopupTyped::Generic(popup) => popup.render(frame, area),
        }
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> PopupEvent {
        match self {
            PopupTyped::Youtube(popup) | PopupTyped::Spotify(popup) => {
                popup.handle_key_events(key_event)
            }
            PopupTyped::Generic(popup) => popup.handle_key_events(key_event),
        }
    }
}
