use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct OntimeEvent {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub event_type: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct OntimeEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub cue: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub colour: String,
    #[serde(default)]
    pub duration: u64,
    #[serde(rename = "timeStart", default)]
    pub time_start: u64,
    #[serde(rename = "timeEnd", default)]
    pub time_end: u64,
    #[serde(default)]
    pub parent: Option<String>,
    // Allow unknown fields to be ignored
    #[serde(flatten)]
    pub _extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct OntimeRundown {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub order: Vec<String>,
    #[serde(rename = "flatOrder", default)]
    pub flat_order: Vec<String>,
    #[serde(default)]
    pub entries: HashMap<String, OntimeEntry>,
    #[serde(default)]
    pub revision: u64,
}
