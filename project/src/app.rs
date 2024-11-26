use std::error;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::{event::{Event, GlobalEvent, GlobalEventData, Platform}, providers::{spotify_provider::SpotifyProvider, youtube_provider::YoutubeProvider}, widgets::{popups::{add_playlist::AddPlaylistPopup, add_song::AddSongPopup, message_popup::MessagePopup, popup::{Popup, PopupEvent, PopupEventTyped, PopupTyped}}, spotify_column::SpotifyColumn, youtube_column::YoutubeColumn}};
use crate::providers::provider_traits::APIProvider;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ActiveBlock {
    SpotifyPlaylistSelector,
    YoutubePlaylistSelector,
    SpotifySongSelector,
    YoutubeSongSelector,
}

/// Application.
#[derive(Debug)]
pub struct App {
    // Is the application running?
    pub running: bool,
    // currently active block widget
    pub active_view: ActiveBlock,

    pub spotify_column: SpotifyColumn,
    pub youtube_column: YoutubeColumn,

    pub popup: PopupTyped,

    pub global_event_sender: mpsc::UnboundedSender<Event>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub async fn new(global_event_sender: mpsc::UnboundedSender<Event>) -> Self {
        let mut app = Self {
            running: true,
            active_view: ActiveBlock::SpotifyPlaylistSelector,
            spotify_column: SpotifyColumn::new(SpotifyProvider::new().await, global_event_sender.clone()),
            youtube_column: YoutubeColumn::new(YoutubeProvider::new().await, global_event_sender.clone()),

            popup: PopupTyped::None,
            global_event_sender,
        };
        app.spotify_column.playlist_selector.active = true;
        app
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn handle_received_data(&mut self, data: GlobalEvent) {
        match data.platform {
            Platform::Spotify => {
                match data.data {
                    GlobalEventData::Playlists(vec) => self.spotify_column.set_playlists(vec),
                    GlobalEventData::Songs(vec) => self.spotify_column.set_songs(vec),
                }
            },
            Platform::Youtube => {
                match data.data {
                    GlobalEventData::Playlists(vec) => self.youtube_column.set_playlists(vec),
                    GlobalEventData::Songs(vec) => self.youtube_column.set_songs(vec),
                }
            },
            Platform::None => todo!(),
        }
    }

    pub async fn handle_key_events(&mut self, key_event: KeyEvent) -> AppResult<()> {
        match self.popup.handle_key_events(key_event).await {
            PopupEventTyped::Youtube(popup_event) => {
                match popup_event {
                    PopupEvent::PopupClose => {
                        self.popup = PopupTyped::None;
                        return Ok(())
                    },
                    PopupEvent::None => {
                        return Ok(())
                    },
                    PopupEvent::Pass => {},
                    PopupEvent::PopupCloseDataPopup(popup) => {
                        self.popup = PopupTyped::Youtube(popup)
                    },
                    PopupEvent::PopupCloseRefresh => {
                        self.popup = PopupTyped::None;
                        self.selective_refresh();
                    },
                }
            },
            PopupEventTyped::Spotify(popup_event) => {
                match popup_event {
                    PopupEvent::PopupClose => {
                        self.popup = PopupTyped::None;
                        return Ok(())
                    },
                    PopupEvent::None => {
                        return Ok(())
                    },
                    PopupEvent::Pass => {},
                    PopupEvent::PopupCloseDataPopup(popup) => {
                        self.popup = PopupTyped::Spotify(popup)
                    },
                    PopupEvent::PopupCloseRefresh => {
                        self.popup = PopupTyped::None;
                        self.selective_refresh();
                    },
                }
            },
        }

        match self.active_view {
            ActiveBlock::SpotifyPlaylistSelector |
            ActiveBlock::SpotifySongSelector => self.spotify_column.handle_key_events(key_event, self.active_view),
            ActiveBlock::YoutubePlaylistSelector |
            ActiveBlock::YoutubeSongSelector => self.youtube_column.handle_key_events(key_event, self.active_view),
        }

        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {self.quit();}
            KeyCode::Char('c') | 
            KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.quit();
                }
            }
            KeyCode::Tab => {
                self.active_view_switch();
            }
            KeyCode::Char('a') | 
            KeyCode::Char('A') => {
                if key_event.modifiers != KeyModifiers::CONTROL {
                    self.handle_item_adding();
                }
            }
            KeyCode::Char('h') => {
                self.selective_refresh();
            }
            KeyCode::Delete => {self.handle_item_removing().await;}
            KeyCode::Left => {
                match self.active_view {
                    ActiveBlock::YoutubePlaylistSelector => {
                        self.popup = PopupTyped::Message(MessagePopup::new("Error".into(), "Not implemented. Please select playlists and move songs.\nYou can use Ctrl-A to move all songs".into()))
                    },
                    ActiveBlock::YoutubeSongSelector => {
                        match (self.youtube_column.song_selector.get_selected_songs(), self.spotify_column.selected_playlist_id.clone()) {
                            (Some(songs), Some(p_id)) => {
                                if !songs.is_empty() {
                                    self.spotify_column.add_found_songs(p_id, songs).await;
                                }
                            },
                            (None, None) |
                            (Some(_), None) => {
                                self.popup = PopupTyped::Message(MessagePopup::new("Error".into(), "You must choose a spotify playlist".into()))
                            },
                            (None, Some(_)) => {
                                self.popup = PopupTyped::Message(MessagePopup::new("Error".into(), "You must choose a songs from youtube playlist (use enter)".into()))
                            }
                        }
                    },
                    _ => ()
                }
            },
            KeyCode::Right => {
                match self.active_view {
                    ActiveBlock::SpotifyPlaylistSelector => todo!(),
                    ActiveBlock::SpotifySongSelector => {
                        match (self.spotify_column.song_selector.get_selected_songs(), self.youtube_column.selected_playlist_id.clone()) {
                            (Some(songs), Some(p_id)) => {
                                if !songs.is_empty() {
                                    self.youtube_column.add_found_songs(p_id, songs).await;
                                }
                            },
                            (None, None) |
                            (Some(_), None) => {
                                self.popup = PopupTyped::Message(MessagePopup::new("Error".into(), "You must choose a youtube playlist".into()))
                            },
                            (None, Some(_)) => {
                                self.popup = PopupTyped::Message(MessagePopup::new("Error".into(), "You must choose a songs from spotify playlist (use enter)".into()))
                            }
                        }
                    },
                    _ => ()
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn handle_item_adding(&mut self) {
        match self.active_view {
            ActiveBlock::SpotifyPlaylistSelector => {
                self.popup = PopupTyped::Spotify(Popup::AddPlaylist(AddPlaylistPopup::new(self.spotify_column.provider.clone())))
            },
            ActiveBlock::YoutubePlaylistSelector => {
                self.popup = PopupTyped::Youtube(Popup::AddPlaylist(AddPlaylistPopup::new(self.youtube_column.provider.clone())))
            },
            ActiveBlock::SpotifySongSelector => {
                if let Some(playlists) = self.spotify_column.playlist_selector.get_selected_songs() {
                    if let Some(playlist) = playlists.first() {
                        if playlist.owned {
                            self.popup = match self.spotify_column.selected_playlist_id.clone() {
                                Some(playlist_id) => {
                                    PopupTyped::Spotify(Popup::AddSong(AddSongPopup::<SpotifyProvider>::new(self.spotify_column.provider.clone(), playlist_id.clone())))
                                },
                                None => {
                                    PopupTyped::Message(MessagePopup::new("Error".into(), "You must choose a spotify playlist".into()))
                                },
                            }
                        } else {
                            self.popup = PopupTyped::Message(MessagePopup::new("Error".into(), "Missing permissions to modify playlist".to_string()))
                        }
                    }
                }
            },
            ActiveBlock::YoutubeSongSelector => {
                self.popup = match self.youtube_column.selected_playlist_id.clone() {
                    Some(playlist_id) => {
                        PopupTyped::Youtube(Popup::AddSong(AddSongPopup::<YoutubeProvider>::new(self.youtube_column.provider.clone(), playlist_id.clone())))
                    },
                    None => {
                        PopupTyped::Message(MessagePopup::new("Error".into(), "You must choose a youtube playlist".into()))
                    },
                }
            },
        }
    }

    pub async fn handle_item_removing(&mut self) {
        match self.active_view {
            ActiveBlock::SpotifyPlaylistSelector |
            ActiveBlock::YoutubePlaylistSelector => {
                self.popup = PopupTyped::Message(MessagePopup::new("Error".into(), "Deleting of playlists not implemented for my own sanity".to_string()));
            },
            ActiveBlock::SpotifySongSelector => {
                if let Some(songs) =  self.spotify_column.song_selector.get_selected_songs() {
                    if let Some(playlist) = self.spotify_column.playlist_selector.get_selected_songs() {
                        if let Some(playlist) = playlist.first() {
                            if playlist.owned && !songs.is_empty() {
                                let song_ids = songs.iter().map(|item| item.id.clone()).collect::<Vec<String>>();
                                self.spotify_column.provider.rem_playlist_song(playlist.id.clone(), song_ids).await;
                                self.spotify_column.song_selector.clear_selected();
                                self.spotify_column.refresh_songs();
                            }
                        }
                    }
                }
            },
            ActiveBlock::YoutubeSongSelector => {
                if let Some(songs) =  self.youtube_column.song_selector.get_selected_songs() {
                    let song_ids = songs.iter().map(|item| {
                        match &item.r#type {
                            crate::types::music_types::RSyncSongProviderData::Youtube(data) => data.playlist_id.as_ref().unwrap().clone(),
                            crate::types::music_types::RSyncSongProviderData::Spotify => panic!("Spotify song in youtube playlist"),
                        }
                    }).collect::<Vec<String>>();
                    
                    self.youtube_column.provider.rem_playlist_song(self.youtube_column.selected_playlist_id.as_ref().unwrap().clone(), song_ids).await;
                    self.spotify_column.song_selector.clear_selected();
                    self.youtube_column.refresh_songs();
                }
            },
        }
    }

    pub fn selective_refresh(&mut self) {
        match self.active_view {
            ActiveBlock::SpotifyPlaylistSelector => self.spotify_column.refresh_playlists(),
            ActiveBlock::YoutubePlaylistSelector => self.youtube_column.refresh_playlists(),
            ActiveBlock::SpotifySongSelector => self.spotify_column.refresh_songs(),
            ActiveBlock::YoutubeSongSelector => self.youtube_column.refresh_songs(),
        }
    }

    pub fn active_view_switch(&mut self) {
        self.active_view = match self.active_view {
            ActiveBlock::SpotifyPlaylistSelector => {
                self.spotify_column.playlist_selector.active = false;
                self.youtube_column.playlist_selector.active = true;
                ActiveBlock::YoutubePlaylistSelector
            },
            ActiveBlock::YoutubePlaylistSelector => {
                self.youtube_column.playlist_selector.active = false;
                self.spotify_column.song_selector.active = true;
                ActiveBlock::SpotifySongSelector
            },
            ActiveBlock::SpotifySongSelector => {
                self.spotify_column.song_selector.active = false;
                self.youtube_column.song_selector.active = true;
                ActiveBlock::YoutubeSongSelector
            },
            ActiveBlock::YoutubeSongSelector => {
                self.youtube_column.song_selector.active = false;   
                self.spotify_column.playlist_selector.active = true;
                ActiveBlock::SpotifyPlaylistSelector
            },
        }
    }
}
