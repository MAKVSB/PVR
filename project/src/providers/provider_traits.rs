use crate::types::music_types::{PlaylistIdWrapper, RSyncPlaylistItem, RSyncSong};

pub trait APIProviderBuilder {
    #[allow(async_fn_in_trait)]
    async fn authorize(&mut self) -> impl APIProvider;
}

pub trait APIProvider {
    #[allow(async_fn_in_trait)]
    async fn new() -> Self;

    #[allow(async_fn_in_trait)]
    async fn get_playlists(&mut self) -> Vec<RSyncPlaylistItem>;

    #[allow(async_fn_in_trait)]
    async fn get_playlist_songs(&mut self, playlist_id: PlaylistIdWrapper) -> Vec<RSyncSong>;

    #[allow(async_fn_in_trait)]
    async fn create_playlist(&mut self, playlist_name: String);

    #[allow(async_fn_in_trait)]
    async fn add_playlist_song(&mut self, playlist_id: PlaylistIdWrapper, song_id: Vec<String>);

    #[allow(async_fn_in_trait)]
    async fn rem_playlist_song(&mut self, playlist_id: PlaylistIdWrapper, song_ids: Vec<String>);

    #[allow(async_fn_in_trait)]
    async fn search(&mut self, query: String, limit: u32) -> Vec<RSyncSong>;
    
    #[allow(async_fn_in_trait)]
    async fn search_list(&mut self, items: Vec<&RSyncSong>) -> Vec<RSyncSong>;
}