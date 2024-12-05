use spotify_rs::{model::{playlist::SimplifiedPlaylist, track::Track, PlayableItem}, AuthCodeClient, RedirectUrl, Token};
use tokio::sync::{mpsc, oneshot};
use std::{collections::HashMap, env, sync::{Arc, Mutex}};
use warp::Filter;
use webbrowser;

use crate::{event::{Event, GlobalEvent, GlobalEventData, GlobalEventDataFullfilness}, types::music_types::{PlaylistIdWrapper, RSyncPlaylistItem, RSyncPlaylistItemProviderData, RSyncPlaylistItemProviderDataSpotify, RSyncSong, RSyncSongProviderData}};

use super::provider_traits::{APIProvider, APIProviderBuilder};

struct LoginQueryParams {
    code: String,
    state: String,
}

pub struct SpotifyProviderBuilder {}
impl SpotifyProviderBuilder {
    #[allow(unused)]
    fn new() -> SpotifyProviderBuilder {
        SpotifyProviderBuilder {}
    }

    async fn new_authorized() -> SpotifyProvider {
        SpotifyProviderBuilder::new().authorize().await
    }
}

impl APIProviderBuilder for SpotifyProviderBuilder {
    #[allow(refining_impl_trait)]
    async fn authorize(&mut self) -> SpotifyProvider {
        // This should match the redirect URI you set in your app's settings
        let var_callback = env::var("SPOTIFY_CALLBACK").unwrap();
        let var_client_id = env::var("SPOTIFY_CLIENT_ID").unwrap();
        let var_client_secret = env::var("SPOTIFY_CLIENT_SECRET").unwrap();

        let redirect_url = RedirectUrl::new(var_callback.to_owned()).unwrap();
        let auto_refresh = true;
        let scopes = vec![
            "app-remote-control", "playlist-read-private", "playlist-read-collaborative", "playlist-modify-private",
            "playlist-modify-public", "user-library-modify", "user-library-read", "user-read-email", "user-read-private",
        ];

        // Redirect the user to this URL to get the auth code and CSRF token
        let (client, url) = AuthCodeClient::new(var_client_id, var_client_secret, scopes, redirect_url, auto_refresh);

        // Step 2: Get the auth token using browser and callback to local server
        webbrowser::open(url.as_str()).unwrap_or_else(|_| panic!("Failed to open browser. Please visit the url {}", url.as_str()));
        
        // Set up a channel to receive the authorization code from the callback
        let (tx, rx) = oneshot::channel::<LoginQueryParams>();

        // Wrap the sender in an Arc<Mutex<Option<Sender>>> to make it safely shareable
        let tx = Arc::new(Mutex::new(Some(tx)));

        // Clone tx to pass it to the route handler
        let tx_filter = warp::any().map(move || Arc::clone(&tx));

        // Define a warp route to capture the authorization code from the callback
        let callback_route = warp::path("callback")
            .and(warp::query::<HashMap<String, String>>())
            .and(tx_filter)
            .map(|query_params: HashMap<String, String>, tx: Arc<Mutex<Option<oneshot::Sender<LoginQueryParams>>>>| {
                if let Some(code) = query_params.get("code") {
                    // If we have a sender, send the code
                    if let Some(tx) = tx.lock().unwrap().take() {
                        let _ = tx.send(LoginQueryParams {code: code.clone(), state: query_params.get("state").unwrap().clone()});
                    }
                }
                // Display a success message to the user
                warp::reply::with_header(
                    warp::reply::html(
                        r#"
                        <!DOCTYPE html>
                        <html>
                        <head>
                            <title>Closing...</title>
                            <script>
                                // Ensure the script runs after the page loads
                                window.onload = function() {
                                    window.close();
                                };
                            </script>
                        </head>
                        <body>
                            <h1>Received data. This window will close shortly.</h1>
                        </body>
                        </html>
                        "#,
                    ),
                    "Content-Type",
                    "text/html",
                )
            });

        // Start the warp server on port 8888
        let server = warp::serve(callback_route).bind(([127, 0, 0, 1], 8989));
        let server_handle = tokio::spawn(server);

        // Wait for either the authorization code or server completion
        let auth_result = tokio::select! {
            _ = server_handle => {
                panic!("Server closed unexpectedly");
            },
            code = rx => {
                code.expect("Failed to receive authorization code")
            },
        };

        // Step 3: Finally, exchange the auth code for an access token
        let client = client.authenticate(auth_result.code, auth_result.state).await.unwrap();
        let owner_name = spotify_rs::get_current_user_profile(&client).await.unwrap().id;

        SpotifyProvider {
            client,
            owner_name,
        }
    }
}

impl RSyncPlaylistItem {
    fn from(item: SimplifiedPlaylist, owner_name: String) -> Self {
        RSyncPlaylistItem {
            collaborative: item.collaborative,
            description: item.description,
            url: item.external_urls.spotify,
            id: PlaylistIdWrapper::Id(item.id),
            name: item.name,
            owned: item.owner.id == owner_name,
            public: item.public.unwrap_or(false),
            tracks: match item.tracks {
                Some(val) => val.total,
                None => 0,
            },
            r#type: RSyncPlaylistItemProviderData::Spotify(RSyncPlaylistItemProviderDataSpotify {
                snapshot_id: item.snapshot_id,
            }),
            
        }
    }
}

impl From<Track> for RSyncSong {
    fn from(track: Track) -> Self {
        RSyncSong {
            artists: track.artists.iter().map(|f| -> String {
                f.name.clone()
            }).collect::<Vec<String>>().join(", "),
            url: track.external_urls.spotify,
            id: track.id,
            name: track.name,
            r#type: RSyncSongProviderData::Spotify
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpotifyProvider {
    client: spotify_rs::AuthCodeClient<Token>,
    owner_name: String,
}

impl APIProvider for SpotifyProvider {
    async fn new() -> Self {
        SpotifyProviderBuilder::new_authorized().await
    }

    async fn get_playlists(&mut self) -> Vec<RSyncPlaylistItem> {
        let mut total: Option<u32> = None;
        let mut offset: u32 = 0;
        let per_request = 10;
        let mut playlists: Vec<RSyncPlaylistItem> = Vec::new();
        playlists.push(RSyncPlaylistItem {
            collaborative: false,
            description: Some("Favourite playlist".into()),
            url: "https://open.spotify.com/collection/tracks".into(),
            id: PlaylistIdWrapper::Liked,
            name: "Favorites".into(),
            owned: true,
            public: false,
            tracks: 0,
            r#type: RSyncPlaylistItemProviderData::Spotify(RSyncPlaylistItemProviderDataSpotify { 
                snapshot_id: "".into() 
            })
        });

        loop {
            let response = spotify_rs::current_user_playlists().limit(per_request).offset(offset).get(&self.client).await.unwrap();
            if total.is_none() {
                total = Some(response.total);
            }
            offset += per_request;

            for playlist in response.items.into_iter().flatten() {
                playlists.push(RSyncPlaylistItem::from(playlist, self.owner_name.clone()));
            }
            if offset > total.unwrap() {
                break;
            }
        }
        playlists
    }

    async fn get_playlist_songs(&mut self, playlist_id: PlaylistIdWrapper, event_sender: Option<(mpsc::UnboundedSender<Event>, u128)>) -> Vec<RSyncSong> {
        match playlist_id {
            PlaylistIdWrapper::Liked => {
                let mut total= None;
                let mut offset: u32 = 0;
                let per_request = 40;
                let mut songs: Vec<RSyncSong> = Vec::new();
        
                loop {
                    let response = spotify_rs::saved_tracks().limit(per_request).offset(offset).get(&self.client).await.unwrap();
                    let mut songs_inner: Vec<RSyncSong> = Vec::new();
                    if total.is_none() {
                        total = Some(response.total);
                    }
                    offset += per_request;
        
                    for playlist_track in response.items {
                        songs_inner.push(playlist_track.unwrap().track.into());
                    }
                    
                    if offset > total.unwrap() {
                        break;
                    }

                    if let Some(event_sender) = event_sender.clone() {
                        event_sender.0.send(Event::DataReceived(event_sender.1, GlobalEvent::Spotify(GlobalEventData::Songs(GlobalEventDataFullfilness::Partial(songs_inner.clone()))))).unwrap();
                    }
                    songs.append(&mut songs_inner);
                }
                songs
            },
            PlaylistIdWrapper::Id(playlist_id) => {
                let mut total: Option<u32> = None;
                let mut offset: u32 = 0;
                let per_request = 20;
                let mut songs: Vec<RSyncSong> = Vec::new();
        
                loop {
                    let response = spotify_rs::playlist_items(&playlist_id).limit(per_request).offset(offset).get(&self.client).await.unwrap();
                    if total.is_none() {
                        total = Some(response.total);
                    }
                    offset += per_request;
        
                    for playlist_track in response.items {
                        if let Some(track_data) = playlist_track {
                            match track_data.track {
                                Some(PlayableItem::Track(track)) => {
                                    songs.push(track.into());
                                },
                                _ => {
                                    // podcasts and are not a part i want to deal with.... sorry
                                    // Also. Some responses are invalid in relation to spotify api definition which is funny :D
                                }
                            }
                        }
                    }
                    if offset > total.unwrap() {
                        break;
                    }
                }
                songs
            },
        }
    }

    async fn create_playlist(&mut self, playlist_name: String) {
        spotify_rs::create_playlist(self.owner_name.clone(), playlist_name).send(&self.client).await.unwrap();
    }

    async fn add_playlist_song(&mut self, playlist_id: PlaylistIdWrapper, song_ids: Vec<String>) {
        match playlist_id {
            PlaylistIdWrapper::Liked => {
                let song_uris: Vec<String> = song_ids.iter().map(|song_id| self.convert_id_to_uri(song_id)).collect();
                spotify_rs::save_tracks(song_uris.as_slice(), &self.client).await.unwrap();
            },
            PlaylistIdWrapper::Id(playlist_id) => {
                let song_uris: Vec<String> = song_ids.iter().map(|song_id| self.convert_id_to_uri(song_id)).collect();
                spotify_rs::add_items_to_playlist(playlist_id, song_uris.as_slice()).send(&self.client).await.unwrap();
            },
        }
    }

    async fn rem_playlist_song(&mut self, playlist_id: PlaylistIdWrapper, song_ids: Vec<String>) {
        let song_uris: Vec<String> = song_ids.iter().map(|song_id| self.convert_id_to_uri(song_id)).collect();
        match playlist_id {
            PlaylistIdWrapper::Liked => {
                spotify_rs::remove_saved_tracks(song_uris.as_slice(), &self.client).await.unwrap();
            },
            PlaylistIdWrapper::Id(playlist_id) => {
                spotify_rs::remove_playlist_items(playlist_id, song_uris.as_slice()).send(&self.client).await.unwrap();
            },
        }
    }

    async fn search(&mut self, query: String, limit: u32) -> Vec<RSyncSong> {
        let items = [spotify_rs::model::search::Item::Track];
        let data = spotify_rs::search(query, &items).limit(limit).get(&self.client).await.unwrap().tracks.unwrap().clone().items;
        let mut songs = Vec::new();
        for track in data {
            songs.push(track.unwrap().into());
        }
        songs
    }
    
    async fn search_list(&mut self, items: Vec<&RSyncSong>) -> Vec<RSyncSong> {
        let mut songs = Vec::new();
        for item in items {
            let found = self.search(format!("{} ({})", item.name, item.artists), 1).await;
            songs.push(found[0].clone());
        }
        songs
    }
}

impl SpotifyProvider {
    pub fn convert_id_to_uri(&mut self, song_id: &str) -> String {
        ["spotify", "track", song_id].join(":")
    }

    pub async fn rem_liked_song(&mut self, song_ids: Vec<String>) {
        let song_uris: Vec<String> = song_ids.iter().map(|song_id| self.convert_id_to_uri(song_id)).collect();
        spotify_rs::remove_saved_tracks(song_uris.as_slice(), &self.client).await.unwrap();
    }
}
