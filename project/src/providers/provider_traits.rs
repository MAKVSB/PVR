use std::future::Future;

use tokio::sync::mpsc;

use crate::{
    event::Event,
    types::music_types::{PlaylistIdWrapper, RSyncPlaylistItem, RSyncSong},
};

pub trait APIProviderBuilder {
    fn authorize(&mut self) -> impl Future<Output = impl APIProvider>;
}

pub trait APIProvider {
    fn new() -> impl Future<Output = impl APIProvider>;

    fn get_playlists(&mut self) -> impl Future<Output = Vec<RSyncPlaylistItem>>;

    fn get_playlist_songs(
        &mut self,
        playlist_id: PlaylistIdWrapper,
        event_sender: Option<(mpsc::UnboundedSender<Event>, u128)>,
    ) -> impl Future<Output = Vec<RSyncSong>>;

    fn create_playlist(&mut self, playlist_name: String) -> impl Future<Output = ()>;

    fn add_playlist_song(
        &mut self,
        playlist_id: PlaylistIdWrapper,
        song_id: Vec<String>,
    ) -> impl Future<Output = ()>;

    fn rem_playlist_song(
        &mut self,
        playlist_id: PlaylistIdWrapper,
        song_ids: Vec<String>,
    ) -> impl Future<Output = ()>;

    fn search(&mut self, query: String, limit: u32) -> impl Future<Output = Vec<RSyncSong>>;

    fn search_list(&mut self, items: Vec<RSyncSong>) -> impl Future<Output = Vec<RSyncSong>>;
}
