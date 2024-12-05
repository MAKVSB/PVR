use std::error;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::{event::{Event, GlobalEvent, GlobalGenericEventData, TransferUpdateEventData}, providers::{spotify_provider::SpotifyProvider, youtube_provider::YoutubeProvider}, widgets::{popups::{add_playlist::AddPlaylistPopup, add_song::AddSongPopup, add_song_selection::AddSongSelectionPopup, message_popup::MessagePopup, popup::{GenericPopup, PlatformPopup, PopupEvent, PopupTyped}}, spotify_column::SpotifyColumn, youtube_column::YoutubeColumn}};
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

    pub popup: Option<PopupTyped>,

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

            popup: None,
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

    pub fn handle_received_data(&mut self, request_id: u128, data: GlobalEvent) {
        match data {
            GlobalEvent::Generic(global_event_data) => {
                match global_event_data {
                    GlobalGenericEventData::TransferUpdate(transfer_update_event_data) => {
                        let title = "Transfering".into();
                        match transfer_update_event_data {
                            TransferUpdateEventData::Searching => {
                                self.popup = Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new(title, "Searching for songs. Please wait".into()))))
                            },
                            TransferUpdateEventData::Updating => {

                                self.popup = Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new(title, "Updating playlist. Please wait".into()))))
                            },
                            TransferUpdateEventData::Finished => {
                                self.popup = None;
                                match self.active_view {
                                    ActiveBlock::SpotifyPlaylistSelector |
                                    ActiveBlock::SpotifySongSelector => {
                                        self.youtube_column.refresh_songs();
                                    },
                                    ActiveBlock::YoutubePlaylistSelector |
                                    ActiveBlock::YoutubeSongSelector => {
                                        self.spotify_column.refresh_songs();
                                    },
                                }
                            },
                        }
                    },
                }
            },
            GlobalEvent::Spotify(global_event_data) => self.spotify_column.handle_received_data(request_id, global_event_data),
            GlobalEvent::Youtube(global_event_data) => self.youtube_column.handle_received_data(request_id, global_event_data),
        }
    }

    pub async fn handle_key_events(&mut self, key_event: KeyEvent) {
        self.check_close_key(key_event);
        if let Some(ref mut popup) = self.popup {
            match popup.handle_key_events(key_event) {
                PopupEvent::PopupClose => {
                    self.popup = None;
                },
                PopupEvent::None => {
                },
                PopupEvent::Pass => {},
                PopupEvent::PopupCloseRefresh => {
                    self.selective_refresh();
                    self.popup = None;
                },
                PopupEvent::PopupCloseData(received_data) => {
                    match &popup {
                        PopupTyped::Spotify(popup) => {
                            match popup {
                                PlatformPopup::AddSong(popup) => {
                                    let found_songs = self.spotify_column.provider.search(received_data, 10).await;
                                    self.popup = Some(PopupTyped::Spotify(PlatformPopup::AddSongSelect(AddSongSelectionPopup::new(found_songs, popup.playlist_id.clone()))));
                                },
                                PlatformPopup::AddSongSelect(popup) => {
                                    self.spotify_column.provider.add_playlist_song(popup.playlist_id.clone(), Vec::from([received_data])).await;
                                    self.selective_refresh();
                                    self.popup = None;
                                },
                                PlatformPopup::AddPlaylist(_) => {
                                    self.spotify_column.provider.create_playlist(received_data).await;
                                    self.spotify_column.refresh_playlists();
                                    self.popup = None;
                                },
                            }
                        },
                        PopupTyped::Youtube(popup) => {
                            match popup {
                                PlatformPopup::AddSong(popup) => {
                                    let found_songs = self.youtube_column.provider.search(received_data, 10).await;
                                    self.popup = Some(PopupTyped::Youtube(PlatformPopup::AddSongSelect(AddSongSelectionPopup::new(found_songs, popup.playlist_id.clone()))));
                                },
                                PlatformPopup::AddSongSelect(popup) => {
                                    self.youtube_column.provider.add_playlist_song(popup.playlist_id.clone(), Vec::from([received_data])).await;
                                    self.selective_refresh();
                                    self.popup = None;
                                },
                                PlatformPopup::AddPlaylist(_) => {
                                    self.youtube_column.provider.create_playlist(received_data).await;
                                    self.youtube_column.refresh_playlists();
                                    self.popup = None;
                                },
                            }
                        },
                        PopupTyped::Generic(popup) => {
                            match popup {
                                GenericPopup::Message(_) => panic!("Not returning any data!"),
                            }
                        }
                    }
                },
            }
        } else {
            match self.active_view {
                ActiveBlock::SpotifyPlaylistSelector |
                ActiveBlock::SpotifySongSelector => self.spotify_column.handle_key_events(key_event, self.active_view),
                ActiveBlock::YoutubePlaylistSelector |
                ActiveBlock::YoutubeSongSelector => self.youtube_column.handle_key_events(key_event, self.active_view),
            }
    
            match key_event.code {
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
                            self.popup = Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new("Error".into(), "Not implemented. Please select playlists and move songs.\nYou can use Ctrl-A to move all songs".into()))))
                        },
                        ActiveBlock::YoutubeSongSelector => {
                            let selected_songs = self.youtube_column.song_selector.get_selected();                            
                            match (selected_songs.is_empty(), self.spotify_column.playlist_selector.get_selected().first()) {
                                (false, Some(playlist)) => {
                                    let p_id = playlist.id.clone();
                                    self.spotify_column.add_found_songs(p_id, selected_songs).await;
                                },
                                (true, None) |
                                (false, None) => {
                                    self.popup = Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new("Error".into(), "You must choose a spotify playlist".into()))))
                                },
                                (true, Some(_)) => {
                                    self.popup = Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new("Error".into(), "You must choose a songs from youtube playlist (use enter)".into()))))
                                }
                            }
                        },
                        _ => ()
                    }
                },
                KeyCode::Right => {
                    match self.active_view {
                        ActiveBlock::SpotifyPlaylistSelector => {
                            self.popup = Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new("Error".into(), "Not implemented. Please select playlists and move songs.\nYou can use Ctrl-A to move all songs".into()))))
                        },
                        ActiveBlock::SpotifySongSelector => {
                            let selected_songs = self.spotify_column.song_selector.get_selected();
                            match (selected_songs.is_empty(), self.youtube_column.playlist_selector.get_selected().first()) {
                                (false, Some(playlist)) => {
                                    let p_id = playlist.id.clone();
                                    self.youtube_column.add_found_songs(p_id, selected_songs).await;
                                },
                                (true, None) |
                                (false, None) => {
                                    self.popup = Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new("Error".into(), "You must choose a youtube playlist".into()))))
                                },
                                (true, Some(_)) => {
                                    self.popup = Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new("Error".into(), "You must choose a songs from spotify playlist (use enter)".into()))))
                                }
                            }
                        },
                        _ => ()
                    }
                }
                _ => {}
            }
        }
    }

    pub fn handle_item_adding(&mut self) {
        self.popup = match self.active_view {
            ActiveBlock::SpotifyPlaylistSelector => {
                Some(PopupTyped::Spotify(PlatformPopup::AddPlaylist(AddPlaylistPopup::new())))
            },
            ActiveBlock::YoutubePlaylistSelector => {
                Some(PopupTyped::Youtube(PlatformPopup::AddPlaylist(AddPlaylistPopup::new())))
            },
            ActiveBlock::SpotifySongSelector => {
                match self.spotify_column.playlist_selector.get_selected().first() {
                    Some(playlist) => {
                        match playlist.owned {
                            true => Some(PopupTyped::Spotify(PlatformPopup::AddSong(AddSongPopup::new(playlist.id.clone())))),
                            false => Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new("Error".into(), "Missing permissions to modify playlist".to_string())))),
                        }
                    },
                    None => Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new("Error".into(), "You must choose a spotify playlist".into())))),
                }
            },
            ActiveBlock::YoutubeSongSelector => {
                match self.youtube_column.playlist_selector.get_selected().first() {
                    Some(playlist) => {
                        match playlist.owned {
                            true => Some(PopupTyped::Youtube(PlatformPopup::AddSong(AddSongPopup::new(playlist.id.clone())))),
                            false => Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new("Error".into(), "Missing permissions to modify playlist".to_string())))),
                        }
                    },
                    None => Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new("Error".into(), "You must choose a spotify playlist".into())))),
                }
            },
        }
    }

    pub async fn handle_item_removing(&mut self) {
        match self.active_view {
            ActiveBlock::SpotifyPlaylistSelector |
            ActiveBlock::YoutubePlaylistSelector => {
                self.popup = Some(PopupTyped::Generic(GenericPopup::Message(MessagePopup::new("Error".into(), "Deleting of playlists not implemented for my own sanity".to_string()))));
            },
            ActiveBlock::SpotifySongSelector => {
                let selected_songs = self.spotify_column.song_selector.get_selected();
                if !selected_songs.is_empty() {
                    if let Some(playlist) = self.spotify_column.playlist_selector.get_selected().first() {
                        if playlist.owned {
                            let song_ids = selected_songs.iter().map(|item| item.id.clone()).collect::<Vec<String>>();
                            self.spotify_column.provider.rem_playlist_song(playlist.id.clone(), song_ids).await;
                            self.spotify_column.song_selector.clear_selected();
                            self.spotify_column.refresh_songs();
                        }
                    }
                }
            },
            ActiveBlock::YoutubeSongSelector => {
                let selected_songs = self.youtube_column.song_selector.get_selected();
                if !selected_songs.is_empty() {
                    if let Some(playlist) = self.youtube_column.playlist_selector.get_selected().first() {
                        if playlist.owned {
                            let song_ids = selected_songs.iter().map(|item| {
                                match &item.r#type {
                                    crate::types::music_types::RSyncSongProviderData::Youtube(data) => data.playlist_id.as_ref().unwrap().clone(),
                                    crate::types::music_types::RSyncSongProviderData::Spotify => panic!("Spotify song in youtube playlist"),
                                }
                            }).collect::<Vec<String>>();
                            
                            self.youtube_column.provider.rem_playlist_song(playlist.id.clone(), song_ids).await;
                            self.youtube_column.song_selector.clear_selected();
                            self.youtube_column.refresh_songs();
                        }
                    }
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

    pub fn check_close_key(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.quit(),
            KeyCode::Char('c') | 
            KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.quit();
                }
            }
            _ => {}
        }
    }
}
