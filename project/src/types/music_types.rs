use ratatui::text::Text;

#[derive(Clone, Debug)]

pub struct RSyncSongProviderDataYoutube {
    pub playlist_id: Option<String>,
}

#[derive(Clone, Debug)]
pub enum RSyncSongProviderData {
    Youtube(RSyncSongProviderDataYoutube),
    Spotify,
}


#[derive(Clone, Debug)]
pub struct RSyncPlaylistItemProviderDataSpotify {
    pub snapshot_id: String
}

#[derive(Clone, Debug)]
pub enum RSyncPlaylistItemProviderData {
    Youtube,
    Spotify(RSyncPlaylistItemProviderDataSpotify),
}

#[derive(Clone, Debug)]
pub enum PlaylistIdWrapper {
    Id(String),
    Liked,
}

#[derive(Clone, Debug)]
pub struct RSyncPlaylistItem {
    pub collaborative: bool,
    pub description: Option<String>,
    pub url: String,
    pub id: PlaylistIdWrapper, // liked songs are "favorite" with string 
    pub name: String,
    pub owned: bool,
    pub public: bool,
    pub tracks: u32,
    pub r#type: RSyncPlaylistItemProviderData
}
impl<'a> Into<Text<'a>> for RSyncPlaylistItem {
    fn into(self) -> Text<'a> {
        let icon = match (self.owned, self.name == "favorites") {
            (true, true) => "♥",
            (true, false) => "  ",
            (false, true) => panic!("Impossible to now own favorites playlist"),
            (false, false) => "🔒",
        };
        [icon.into(), self.name].join(" ").into()
    }
}

#[derive(Clone, Debug)]
pub struct RSyncSong {
    pub artists: String,
    pub url: String,
    pub id: String,
    pub name: String,
    pub r#type: RSyncSongProviderData,
}

impl<'a> Into<Text<'a>> for RSyncSong {
    fn into(self) -> Text<'a> {
        format!("{} ({})", self.name, self.artists).into()
    }
}