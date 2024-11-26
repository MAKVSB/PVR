use crossterm::event::KeyEvent;

use crate::{providers::{provider_traits::APIProvider, spotify_provider::SpotifyProvider, youtube_provider::YoutubeProvider}, widgets::popups::{add_playlist::AddPlaylistPopup, add_song::AddSongPopup, add_song_selection::AddSongSelectionPopup, message_popup::{MessagePopup, MessagePopupEvent}}};

#[derive(Debug)]

pub enum PopupEvent<T>
where 
    T: APIProvider + Clone
{
    PopupClose,
    PopupCloseRefresh,
    PopupCloseDataPopup(Popup<T>),
    None,
    Pass,
}

#[derive(Debug)]

pub enum PopupEventTyped {
    Youtube(PopupEvent<YoutubeProvider>),
    Spotify(PopupEvent<SpotifyProvider>)
}

#[derive(Debug)]
pub enum Popup<T>
where 
    T: APIProvider + Clone
{
    AddSong(AddSongPopup<T>),
    AddSongSelect(AddSongSelectionPopup<T>),
    AddPlaylist(AddPlaylistPopup<T>)
}
impl<T> Popup<T>
where 
    T: APIProvider + Clone
{
    pub fn render(&mut self, frame: &mut ratatui::Frame<'_>, area:ratatui::prelude::Rect ) {
        match self {
            Popup::AddSong(add_song_popup) => add_song_popup.render(frame, area),
            Popup::AddSongSelect(add_song_popup) => add_song_popup.render(frame, area),
            Popup::AddPlaylist(add_playlist_popup) => add_playlist_popup.render(frame, area),
        }
    }

    pub async fn handle_key_events(&mut self, key_event: KeyEvent) -> PopupEvent<T> {
        match self {
            Popup::AddSong(add_song_popup) => add_song_popup.handle_key_events(key_event).await,
            Popup::AddSongSelect(add_song_popup) => add_song_popup.handle_key_events(key_event).await,
            Popup::AddPlaylist(add_playlist_popup) => add_playlist_popup.handle_key_events(key_event).await,
        }
    }
}

#[derive(Debug)]
pub enum PopupTyped {
    Spotify(Popup<SpotifyProvider>),
    Youtube(Popup<YoutubeProvider>),
    Message(MessagePopup),
    None,
}
impl PopupTyped{
    pub fn render(&mut self, frame: &mut ratatui::Frame<'_>, area:ratatui::prelude::Rect ) {
        match self {
            PopupTyped::Youtube(add_song_popup) => add_song_popup.render(frame, area),
            PopupTyped::Spotify(add_song_popup) => add_song_popup.render(frame, area),
            PopupTyped::Message(message_popup) => message_popup.render(frame, area),
            PopupTyped::None => {},
        }
    }

    pub fn is_none(&mut self) -> bool {
        matches!(self,PopupTyped::None)
    }

    pub async fn handle_key_events(&mut self, key_event: KeyEvent) -> PopupEventTyped {
        match self {
            PopupTyped::Youtube(add_song_popup) => PopupEventTyped::Youtube(add_song_popup.handle_key_events(key_event).await),
            PopupTyped::Spotify(add_song_popup) => PopupEventTyped::Spotify(add_song_popup.handle_key_events(key_event).await),
            PopupTyped::Message(message_popup) => {
                let res = message_popup.handle_key_events(key_event);
                match res {
                    MessagePopupEvent::None => PopupEventTyped::Spotify(PopupEvent::None),
                    MessagePopupEvent::Pass => PopupEventTyped::Spotify(PopupEvent::Pass),
                    MessagePopupEvent::PopupClose => PopupEventTyped::Spotify(PopupEvent::PopupClose),
                }
            },
            PopupTyped::None => PopupEventTyped::Spotify(PopupEvent::Pass),
        }
    }
}