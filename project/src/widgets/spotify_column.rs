use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use tokio::sync::mpsc;

use crate::{
    app::ActiveBlock,
    event::{
        Event, GlobalEvent, GlobalEventData, GlobalEventDataFullfilness, GlobalGenericEventData,
        TransferUpdateEventData,
    },
    providers::{provider_traits::APIProvider, spotify_provider::SpotifyProvider},
    types::{
        music_types::{PlaylistIdWrapper, RSyncPlaylistItem, RSyncSong},
        playlist_selector_key_event_response::SelectorKeyEventResponse,
    },
};

use super::{playlist_selector::PlaylistSelector, song_selector::SongSelector};

#[derive(Debug)]
pub struct SpotifyColumn {
    pub provider: SpotifyProvider,
    pub playlist_selector: PlaylistSelector,
    pub song_selector: SongSelector,
    render_rows: Layout,
    global_event_sender: mpsc::UnboundedSender<Event>,
    last_songs_request_id: u128,
    last_playlists_request_id: u128,
}
impl SpotifyColumn {
    pub fn new(
        provider: SpotifyProvider,
        global_event_sender: mpsc::UnboundedSender<Event>,
    ) -> Self {
        let mut s = Self {
            playlist_selector: PlaylistSelector::new("Spotify playlists".into()),
            song_selector: SongSelector::new("Playlist songs".into()),
            provider,
            render_rows: Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(60)]),
            global_event_sender,
            last_playlists_request_id: 0,
            last_songs_request_id: 0,
        };
        s.refresh_playlists();
        s
    }

    pub fn refresh_songs(&mut self) {
        self.song_selector.set_items(None);
        if let Some(playlist) = self.playlist_selector.get_selected().first() {
            self.song_selector.set_loading();
            let p_id = playlist.id.clone();
            let mut provider_clone = self.provider.clone();
            let event_sender = self.global_event_sender.clone();
            let request_id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            self.last_songs_request_id = request_id;
            tokio::spawn(async move {
                let a = provider_clone
                    .get_playlist_songs(p_id, Some((event_sender.clone(), request_id)))
                    .await;
                event_sender.send(Event::DataReceived(
                    request_id,
                    GlobalEvent::Spotify(GlobalEventData::Songs(GlobalEventDataFullfilness::Full(
                        a,
                    ))),
                ))
            });
        }
    }

    pub fn append_songs(&mut self, items: Vec<RSyncSong>) {
        self.song_selector.append_items(items);
    }

    pub fn set_songs(&mut self, items: Option<Vec<RSyncSong>>) {
        self.song_selector.set_items(items);
        self.song_selector.clear_selected();
    }

    pub fn refresh_playlists(&mut self) {
        self.playlist_selector.set_items(None);
        self.playlist_selector.set_loading();
        let mut provider_clone = self.provider.clone();
        let event_sender = self.global_event_sender.clone();
        let request_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        self.last_playlists_request_id = request_id;
        tokio::spawn(async move {
            let a = provider_clone.get_playlists().await;
            event_sender.send(Event::DataReceived(
                request_id,
                GlobalEvent::Spotify(GlobalEventData::Playlists(
                    GlobalEventDataFullfilness::Full(a),
                )),
            ))
        });
    }

    pub fn append_playlists(&mut self, items: Vec<RSyncPlaylistItem>) {
        self.playlist_selector.append_items(items);
    }

    pub fn set_playlists(&mut self, items: Option<Vec<RSyncPlaylistItem>>) {
        self.playlist_selector.set_items(items);
        self.playlist_selector.clear_selected();
    }

    pub async fn add_found_songs(&mut self, p_id: PlaylistIdWrapper, songs: Vec<&RSyncSong>) {
        let mut provider_clone = self.provider.clone();
        let event_sender = self.global_event_sender.clone();
        let request_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let songs = songs.into_iter().map(|i| (*i).clone()).collect();

        tokio::spawn(async move {
            event_sender
                .send(Event::DataReceived(
                    request_id,
                    GlobalEvent::Generic(GlobalGenericEventData::TransferUpdate(
                        TransferUpdateEventData::Searching,
                    )),
                ))
                .unwrap();
            let found_songs = provider_clone.search_list(songs).await;
            event_sender
                .send(Event::DataReceived(
                    request_id,
                    GlobalEvent::Generic(GlobalGenericEventData::TransferUpdate(
                        TransferUpdateEventData::Updating,
                    )),
                ))
                .unwrap();
            let song_ids = found_songs
                .iter()
                .map(|item| item.id.clone())
                .collect::<Vec<String>>();
            provider_clone.add_playlist_song(p_id, song_ids).await;
            event_sender
                .send(Event::DataReceived(
                    request_id,
                    GlobalEvent::Generic(GlobalGenericEventData::TransferUpdate(
                        TransferUpdateEventData::Finished,
                    )),
                ))
                .unwrap();
        });
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [playlist_selection_area, song_selection_area] = self.render_rows.areas(area);
        self.playlist_selector
            .render(frame, playlist_selection_area);
        self.song_selector.render(frame, song_selection_area);
    }

    pub fn handle_received_data(&mut self, request_id: u128, data: GlobalEventData) {
        match data {
            GlobalEventData::Playlists(event_data) => {
                if request_id == self.last_playlists_request_id {
                    match event_data {
                        GlobalEventDataFullfilness::Partial(vec) => self.append_playlists(vec),
                        GlobalEventDataFullfilness::Full(vec) => self.set_playlists(Some(vec)),
                    }
                }
            }

            GlobalEventData::Songs(event_data) => {
                if request_id == self.last_songs_request_id {
                    match event_data {
                        GlobalEventDataFullfilness::Partial(vec) => self.append_songs(vec),
                        GlobalEventDataFullfilness::Full(vec) => self.set_songs(Some(vec)),
                    }
                }
            }
        }
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent, active_block: ActiveBlock) {
        match active_block {
            ActiveBlock::SpotifyPlaylistSelector => {
                match self.playlist_selector.handle_key_events(key_event) {
                    SelectorKeyEventResponse::Selected(_) => {
                        self.refresh_songs();
                    }
                    SelectorKeyEventResponse::Refresh => {
                        self.refresh_playlists();
                    }
                    SelectorKeyEventResponse::None => (),
                    SelectorKeyEventResponse::Pass => {}
                };
            }
            ActiveBlock::SpotifySongSelector => {
                match self.song_selector.handle_key_events(key_event) {
                    SelectorKeyEventResponse::Selected(_) => (),
                    SelectorKeyEventResponse::Refresh => {
                        self.refresh_songs();
                    }
                    SelectorKeyEventResponse::None => (),
                    SelectorKeyEventResponse::Pass => {}
                };
            }
            _ => (),
        };
    }
}
