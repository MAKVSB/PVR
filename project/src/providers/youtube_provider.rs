use std::{env, fmt::Debug};

use google_youtube3::{api::{Playlist, PlaylistItem, PlaylistItemSnippet, PlaylistSnippet, PlaylistStatus, ResourceId, SearchResult, Video}, hyper_rustls, hyper_util, yup_oauth2::{self, authenticator_delegate::InstalledFlowDelegate}, YouTube};
use tokio::sync::mpsc;

use crate::{event::{Event, GlobalEvent, GlobalEventData, GlobalEventDataFullfilness}, types::music_types::{PlaylistIdWrapper, RSyncPlaylistItem, RSyncPlaylistItemProviderData, RSyncSong, RSyncSongProviderData, RSyncSongProviderDataYoutube}};

use super::provider_traits::{APIProvider, APIProviderBuilder};

pub struct YoutubeProviderBuilder {}
impl YoutubeProviderBuilder {
    #[allow(unused)]
    fn new() -> YoutubeProviderBuilder {
        YoutubeProviderBuilder {}
    }

    async fn new_authorized() -> YoutubeProvider {
        YoutubeProviderBuilder::new().authorize().await
    }
}

struct YupOauthDelegate {}
impl InstalledFlowDelegate for YupOauthDelegate {
    fn redirect_uri(&self) -> Option<&str> {
        None
    }

    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        need_code: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(self.open_browser(url, need_code))
    }
}


impl YupOauthDelegate {
    async fn open_browser(&self, url: &str, need_code: bool) -> Result<String, String> {
        use tokio::io::AsyncBufReadExt;
        if need_code {
            webbrowser::open(url).unwrap_or_else(|_| panic!("Failed to open browser. Please visit the url {}", url));
            let mut user_input = String::new();
            tokio::io::BufReader::new(tokio::io::stdin())
                .read_line(&mut user_input)
                .await
                .map_err(|e| format!("couldn't read code: {}", e))?;
            // remove trailing whitespace.
            user_input.truncate(user_input.trim_end().len());
            Ok(user_input)
        } else {
            webbrowser::open(url).unwrap_or_else(|_| panic!("Failed to open browser. Please visit the url {}", url));
            Ok(String::new())
        }
    }
}

impl APIProviderBuilder for YoutubeProviderBuilder {
    #[allow(refining_impl_trait)]
    async fn authorize(&mut self) -> YoutubeProvider {
        let secret: yup_oauth2::ApplicationSecret = yup_oauth2::ApplicationSecret {
            client_secret: env::var("YOUTUBE_CLIENT_SECRET").unwrap(),
            client_id: env::var("YOUTUBE_CLIENT_ID").unwrap(),
            project_id: Some(env::var("YOUTUBE_PROJECT_ID").unwrap()),
            auth_uri: env::var("YOUTUBE_AUTH_URI").unwrap(),
            token_uri: env::var("YOUTUBE_TOKEN_URI").unwrap(),
            auth_provider_x509_cert_url: Some(env::var("YOUTUBE_CERTS").unwrap()),
            redirect_uris: vec!["http://localhost".into()],
            ..Default::default()
        };
    
        let custom_flow_delegate = YupOauthDelegate {};
        let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
            secret,
            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .flow_delegate(Box::new(custom_flow_delegate))
        .persist_tokens_to_disk("tokencache.json")
        .build()
        .await
        .unwrap();
    
        //this will for some BS reason just printy
        let _ = auth.token(&["https://www.googleapis.com/auth/youtube"]).await.unwrap();


        let yt_client = hyper_util::client::legacy::Client::builder(
            hyper_util::rt::TokioExecutor::new()
        )
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .unwrap()
                .https_or_http()
                .enable_http1()
                .build()
        );
        
        let client = YouTube::new(yt_client, auth);
        let liked_playlist_id = client.channels()
            .list(&vec!["contentDetails".into()])
            .mine(true)
            .doit().await.unwrap().1.items.unwrap().first().unwrap().content_details.clone().unwrap().related_playlists.unwrap().likes.unwrap();

        YoutubeProvider {
            client,
            liked_playlist_id,
        }
    }
}

#[derive(Clone)]
pub struct YoutubeProvider {
    client: YouTube<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
    liked_playlist_id: String,
}
impl Debug for YoutubeProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YoutubeProvider").finish()
    }
}

impl From<Playlist> for RSyncPlaylistItem {
    fn from(item: Playlist) -> Self {
        RSyncPlaylistItem {
            collaborative: false,
            description: item.snippet.clone().unwrap().description,
            url: format!("https://music.youtube.com/playlist?list={}", item.id.as_ref().unwrap()),
            id: PlaylistIdWrapper::Id(item.id.unwrap()),
            name: item.snippet.as_ref().unwrap().title.as_ref().unwrap().clone(),
            owned: item.snippet.unwrap().channel_id.unwrap() == "UC_ACaQ9yyt3iLSDWbg9SO9g",
            public: item.status.unwrap().privacy_status.unwrap() == "private",
            tracks: item.content_details.unwrap().item_count.unwrap(),
            r#type: RSyncPlaylistItemProviderData::Youtube, 
        }
    }
}

impl From<PlaylistItem> for RSyncSong {
    fn from(track: PlaylistItem) -> Self {
        let snippet = track.snippet.unwrap();
        let artist_name = snippet.video_owner_channel_title.clone().unwrap_or_default();
        RSyncSong {
            artists: artist_name.strip_suffix(" - Topic").unwrap_or(&artist_name).to_string(),
            url: format!("https://music.youtube.com/watch?v={}", snippet.resource_id.as_ref().unwrap().video_id.as_ref().unwrap()),
            id: snippet.resource_id.unwrap().video_id.unwrap(),
            name: snippet.title.unwrap(),
            r#type: RSyncSongProviderData::Youtube(RSyncSongProviderDataYoutube {
                playlist_id: track.id,
            })
        }
    }
}

impl From<SearchResult> for RSyncSong {
    fn from(track: SearchResult) -> Self {
        let snippet = track.snippet.unwrap();
        let artist_name = snippet.channel_title.clone().unwrap_or_default();
        RSyncSong {
            artists: artist_name.strip_suffix(" - Topic").unwrap_or(&artist_name).to_string(),
            url: format!("https://music.youtube.com/watch?v={}", track.id.as_ref().unwrap().video_id.clone().unwrap()),
            id: track.id.unwrap().video_id.unwrap(),
            name: snippet.title.unwrap(),
            r#type: RSyncSongProviderData::Youtube(RSyncSongProviderDataYoutube {
                playlist_id: None,
            })
        }
    }
}

impl APIProvider for YoutubeProvider {
    async fn new() -> Self {
        YoutubeProviderBuilder::new_authorized().await
    }

    async fn get_playlists(&mut self) -> Vec<RSyncPlaylistItem> {
        let mut next_page_token: Option<String> = Some("".into());
        let mut playlists: Vec<RSyncPlaylistItem> = Vec::new();

        playlists.push(RSyncPlaylistItem {
            collaborative: false,
            description: Some("Favourite playlist".into()),
            url: format!("https://www.youtube.com/playlist?list={}", self.liked_playlist_id.clone()).into(),
            id: PlaylistIdWrapper::Liked,
            name: "Favorites".into(),
            owned: true,
            public: false,
            tracks: 0,
            r#type: RSyncPlaylistItemProviderData::Youtube
        });

        loop {
            if next_page_token.is_none() {
                break;
            }

            let result = self.client
                        .playlists()
                        .list(&vec!["snippet".into(), "contentDetails".into(), "status".into()])
                        .page_token(next_page_token.unwrap().as_str())
                        .mine(true)
                        .doit().await;
            let result_body = result.unwrap().1;
            next_page_token = result_body.next_page_token.clone();

            for playlist in result_body.items.unwrap() {
                playlists.push(playlist.into());
            }
        }
        playlists
    }

    async fn get_playlist_songs(&mut self, playlist_id: PlaylistIdWrapper, event_sender: Option<(mpsc::UnboundedSender<Event>, u128)> ) -> Vec<RSyncSong> {
        match playlist_id {
            PlaylistIdWrapper::Liked => {
                self.get_playlist_songs_inner(&self.liked_playlist_id.clone(), event_sender).await
            },
            PlaylistIdWrapper::Id(playlist_id) => {
                self.get_playlist_songs_inner(&playlist_id, event_sender).await
            },
        }
    }

    async fn create_playlist(&mut self, playlist_name: String) {
        self.client.playlists().insert(Playlist {
            snippet: Some(PlaylistSnippet {
                tags: Some(["RustSync".to_string()].into()),
                title: Some(playlist_name),
                ..Default::default()
            }),
            status: Some(PlaylistStatus {
                privacy_status: Some("private".into()),
            }),
            ..Default::default()
        }).doit().await.unwrap();
    }

    async fn add_playlist_song(&mut self, playlist_id: PlaylistIdWrapper, song_id: Vec<String>) {
        match playlist_id {
            PlaylistIdWrapper::Id(p_id) => {
                for id in song_id {
                    self.client.playlist_items().insert(PlaylistItem {
                        snippet: Some(PlaylistItemSnippet {
                            playlist_id: Some(p_id.clone()),
                            resource_id: Some(ResourceId {
                                kind: Some("youtube#video".into()),
                                video_id: Some(id),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }).doit().await.unwrap();
                }
            },
            PlaylistIdWrapper::Liked => {
                for id in song_id {
                    self.client.videos().rate(id.as_str(), "like").doit().await.unwrap();
                }
            },
        }






    }

    async fn search(&mut self, query: String, limit: u32) -> Vec<RSyncSong> {       
        let search_data = self.client.search()
            .list(&vec!["snippet".into()])
            .q(query.as_str())
            .video_category_id("10")
            .add_type("video")
            .doit().await.unwrap().1.items.unwrap();

        let mut songs = Vec::new();

        //all this second request mess for almost nothing
        let song_ids: Vec<String> = search_data
            .iter()  
            .map(|data: &SearchResult| -> String {
                data.id.clone().unwrap().video_id.unwrap()
            }).collect();

        if song_ids.is_empty() {
            return songs;
        }

        let detailed_song_data = self.get_detailed_video_data(song_ids).await;

        for (song, detailed_song) in search_data.iter().zip(detailed_song_data) {
            if detailed_song.snippet.unwrap().category_id.unwrap() == "10" {
                //all this mess up here just to get the category number to filter just the music cause youtube-music api does not have public access 
                songs.push(song.clone().into());
            }
        }

        if songs.len() as u32 > limit {
            songs = songs[0..((limit+1) as usize)].to_vec();
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
    
    async fn rem_playlist_song(&mut self, _playlist_id: PlaylistIdWrapper, song_ids: Vec<String>) {
        for id in song_ids {
            self.client.playlist_items().delete(&id).doit().await.unwrap();
        }
    }
}

impl YoutubeProvider {
    async fn get_playlist_songs_inner(&mut self, playlist_id: &String, event_sender: Option<(mpsc::UnboundedSender<Event>, u128)>) -> Vec<RSyncSong> {
        let mut next_page_token: Option<String> = Some("".into());
        let mut songs: Vec<RSyncSong> = Vec::new();

        loop {
            if next_page_token.is_none() {
                break;
            }

            let mut songs_inner: Vec<RSyncSong> = Vec::new();
            let result_body = self.client
                        .playlist_items()
                        .list(&vec!["snippet".into(), "contentDetails".into(), "status".into()])
                        .playlist_id(playlist_id.as_str())
                        .page_token(next_page_token.unwrap().as_str())
                        .doit().await
                        .unwrap().1;
            next_page_token = result_body.next_page_token.clone();

            //all this second request mess for almost nothing
            let song_ids: Vec<String> = result_body.items.as_ref().unwrap()
                .iter()
                .map(|data| -> String {
                    data.snippet.clone().unwrap()
                        .resource_id.unwrap()
                        .video_id.unwrap()
                        .clone()
                }).collect();

            if song_ids.is_empty() {
                return songs;
            }

            let detailed_song_data = self.get_detailed_video_data(song_ids).await;

            for (song, detailed_song) in result_body.items.as_ref().unwrap().iter().zip(detailed_song_data) {
                //all this mess up here just to get the category number to filter just the music cause youtube-music api does not have public access
                if detailed_song.snippet.unwrap().category_id.unwrap() == "10" { 
                    songs_inner.push(song.clone().into());
                }
            }

            if let Some(event_sender) = event_sender.clone() {
                event_sender.0.send(Event::DataReceived(event_sender.1, GlobalEvent::Youtube(GlobalEventData::Songs(GlobalEventDataFullfilness::Partial(songs_inner.clone()))))).unwrap();
            }
            songs.append(&mut songs_inner);
        }

        songs
    }

    pub async fn get_detailed_video_data(&mut self, song_ids: Vec<String>) -> Vec<Video>{
        //all this second request mess for almost nothing
        let mut detailed_song_data_request = self.client.videos().list(&vec!["snippet".into()]);

        for song_id in song_ids {
            detailed_song_data_request = detailed_song_data_request.add_id(&song_id.clone());
        }
        detailed_song_data_request.doit().await.unwrap().1.items.unwrap()
    }
}