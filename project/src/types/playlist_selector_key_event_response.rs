use super::music_types::PlaylistIdWrapper;

#[derive(Debug)]
pub enum SelectorKeyEventResponse {
    Selected(PlaylistIdWrapper),
    Refresh,
    None,
    Pass,
}