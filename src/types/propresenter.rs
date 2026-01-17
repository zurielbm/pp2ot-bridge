use serde::Deserialize;

/// ProPresenter playlist response structure
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct PlaylistResponse {
    pub id: Dictionary,
    pub items: Vec<PlaylistItem>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct PlaylistItem {
    pub id: Dictionary,
    #[serde(rename = "type")]
    pub item_type: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Dictionary {
    pub uuid: String,
    pub name: String,
    pub index: usize,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct PlaylistInfo {
    pub id: Dictionary,
}
