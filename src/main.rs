use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/settings")]
    Settings {},
    #[route("/")]
    Formatter {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        // Preload fonts
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.gstatic.com",
            crossorigin: "true",
        }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;700&display=swap",
        }
        Router::<Route> {}
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        nav { id: "navbar",
            div { class: "nav-brand",
                span { class: "accent", "PP2OT" }
                " BRIDGE"
            }
            div { class: "nav-links",
                Link {
                    to: Route::Formatter {},
                    class: "nav-item",
                    active_class: "active",
                    "FORMATTER"
                }
                Link {
                    to: Route::Settings {},
                    class: "nav-item",
                    active_class: "active",
                    "SETTINGS"
                }
            }
        }
        div { id: "app-container", Outlet::<Route> {} }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct AppSettings {
    pp_host: String,
    pp_port: String,
    ot_host: String,
    ot_port: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            pp_host: "localhost".to_string(),
            pp_port: "1025".to_string(),
            ot_host: "localhost".to_string(),
            ot_port: "4001".to_string(),
        }
    }
}

impl AppSettings {
    fn load() -> Self {
        if let Ok(contents) = std::fs::read_to_string("settings.json") {
            if let Ok(settings) = serde_json::from_str(&contents) {
                return settings;
            }
        }
        Self::default()
    }

    fn save(&self) -> std::io::Result<()> {
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write("settings.json", contents)
    }
}

/// Home page - Bridge Control Center
#[derive(Debug, serde::Deserialize, PartialEq, Clone)]
struct PlaylistResponse {
    id: Dictionary,
    items: Vec<PlaylistItem>,
}

#[derive(Debug, serde::Deserialize, PartialEq, Clone)]
struct PlaylistItem {
    id: Dictionary,
    #[serde(rename = "type")]
    item_type: String,
}

#[derive(Debug, serde::Deserialize, PartialEq, Clone)]
struct Dictionary {
    uuid: String,
    name: String,
    index: usize,
}



/// Settings page
#[component]
fn Settings() -> Element {
    // Initialize with loaded settings
    let mut settings = use_signal(AppSettings::load);
    let mut save_status = use_signal(|| "");

    rsx! {
        div { class: "page-container settings-view",
            h1 { "CONFIGURATION" }

            div { class: "settings-grid",
                // ProPresenter Config
                div { class: "settings-card",
                    div { class: "card-header", "PROPRESENTER SOURCE" }
                    div { class: "input-group",
                        label { "Host Address" }
                        input {
                            value: "{settings.read().pp_host}",
                            oninput: move |e| settings.write().pp_host = e.value(),
                        }
                    }
                    div { class: "input-group",
                        label { "Port" }
                        input {
                            value: "{settings.read().pp_port}",
                            oninput: move |e| settings.write().pp_port = e.value(),
                        }
                    }
                }

                // OnTime Config
                div { class: "settings-card",
                    div { class: "card-header", "ONTIME DESTINATION" }
                    div { class: "input-group",
                        label { "Host Address" }
                        input {
                            value: "{settings.read().ot_host}",
                            oninput: move |e| settings.write().ot_host = e.value(),
                        }
                    }
                    div { class: "input-group",
                        label { "Port" }
                        input {
                            value: "{settings.read().ot_port}",
                            oninput: move |e| settings.write().ot_port = e.value(),
                        }
                    }
                }

                // Sync Settings
                div { class: "settings-card full-width",
                    div { class: "card-header", "SYNC SETTINGS" }
                    div { class: "setting-row",
                        label { "Sync Interval (ms)" }
                        input { r#type: "number", value: "500" }
                    }
                    div { class: "setting-row",
                        label { "Auto-start on launch" }
                        input { r#type: "checkbox" }
                    }
                }

                div {
                    class: "settings-card full-width",
                    style: "background: transparent; border: none; box-shadow: none;",
                    button {
                        class: "btn-secondary",
                        style: "background: var(--accent-bridge); color: #000; border-color: var(--accent-bridge); font-weight: 800;",
                        onclick: move |_| {
                            match settings.read().save() {
                                Ok(_) => save_status.set("Configuration Saved!"),
                                Err(e) => save_status.set("Failed to save!"),
                            }
                        },
                        "SAVE CONFIGURATION"
                    }
                    div { style: "text-align: center; color: var(--color-success); margin-top: 10px; height: 20px;",
                        "{save_status}"
                    }
                }
            }
        }
    }
}

// ============================================================================
// TIME FORMATTER
// ============================================================================

/// Parse a duration string (HH:MM:SS) to milliseconds
fn parse_duration_to_ms(duration: &str) -> u64 {
    let parts: Vec<&str> = duration.split(':').collect();
    if parts.len() == 3 {
        let hours: u64 = parts[0].parse().unwrap_or(0);
        let minutes: u64 = parts[1].parse().unwrap_or(0);
        let seconds: u64 = parts[2].parse().unwrap_or(0);
        (hours * 3600 + minutes * 60 + seconds) * 1000
    } else {
        300000 // Default 5 minutes
    }
}

/// Format milliseconds to duration string (HH:MM:SS)
fn format_ms_to_duration(ms: u64) -> String {
    let seconds = ms / 1000;
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

#[derive(Debug, Clone, PartialEq)]
struct TimedEntry {
    item_id: String,
    name: String,
    item_type: String,
    duration: String,
    end_time: String,
    count_to_end: bool,
    // Optional insertion index for UI placement (0 = start, n = after n-th existing entry)
    insertion_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
struct FormatterGroup {
    id: String,
    name: String,
    color: String,
    entries: Vec<TimedEntry>,
    collapsed: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum InsertionMode {
    After,
    Into, // For groups
}

/// Unified item type for the formatter - can be standalone or a group
#[derive(Debug, Clone, PartialEq)]
enum FormatterItem {
    Standalone(TimedEntry),
    Group {
        id: String,
        name: String,
        color: String,
        entries: Vec<TimedEntry>,
        collapsed: bool,
    },
    Reference {
        id: String,
        title: String,
        item_type: String,
        mode: InsertionMode,
    },
}

/// Formatter page - Time Formatter System
#[derive(Debug, serde::Deserialize, PartialEq, Clone)]
struct PlaylistInfo {
    id: Dictionary,
}

#[derive(Debug, serde::Deserialize, PartialEq, Clone)]
struct OntimeEvent {
    id: String,
    title: String,
    #[serde(rename = "type")]
    event_type: String,
}

#[derive(Debug, serde::Deserialize, Clone, PartialEq)]
struct OntimeEntry {
    id: String,
    #[serde(rename = "type")]
    entry_type: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    cue: String,
    #[serde(default)]
    note: String,
    #[serde(default)]
    colour: String,
    #[serde(default)]
    duration: u64,
    #[serde(rename = "timeStart", default)]
    time_start: u64,
    #[serde(rename = "timeEnd", default)]
    time_end: u64,
    #[serde(default)]
    parent: Option<String>,
    // Allow unknown fields to be ignored
    #[serde(flatten)]
    _extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, serde::Deserialize, Clone, PartialEq)]
struct OntimeRundown {
    id: String,
    title: String,
    #[serde(default)]
    order: Vec<String>,
    #[serde(rename = "flatOrder", default)]
    flat_order: Vec<String>,
    #[serde(default)]
    entries: std::collections::HashMap<String, OntimeEntry>,
    #[serde(default)]
    revision: u64,
}

#[component]
fn Formatter() -> Element {
    let mut playlist_name = use_signal(|| String::new());
    // Unified list of items (standalone entries and groups)
    let mut formatter_items = use_signal(|| Vec::<FormatterItem>::new());
    // None = standalone mode (append to end), Some(idx) = add inside group at that index
    let mut selected_group_idx = use_signal(|| Option::<usize>::None);
    let mut logs = use_signal(|| vec![
        format!("[{}] System Ready", chrono::Local::now().format("%H:%M:%S"))
    ]);

    let mut add_log = move |msg: String| {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        logs.write().push(format!("[{}] {}", timestamp, msg));
    };
    
    // Fetch all available playlists
    let playlists_resource = use_resource(|| async move {
        let settings = AppSettings::load();
        let url = format!("http://{}:{}/v1/playlists", settings.pp_host, settings.pp_port);
        let client = reqwest::Client::new();
        let res = client.get(&url).header("accept", "application/json").send().await;
        
        match res {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Vec<PlaylistInfo>>().await {
                        Ok(playlists) => Ok(playlists),
                        Err(e) => Err(format!("Parse error: {}", e))
                    }
                } else { Err(format!("API Error: {}", response.status())) }
            },
            Err(_) => Err("Connection failed".to_string())
        }
    });

    // Fetch existing Ontime rundown for insertion selection and duplicate checking
    let mut ontime_resource = use_resource(|| async move {
        let settings = AppSettings::load();
        let base_url = format!("http://{}:{}", settings.ot_host, settings.ot_port);
        let client = reqwest::Client::new();
        
        // Fetch from current rundown (more reliable than default)
        let url = format!("{}/data/rundowns/current", base_url);
        let res = client.get(&url).header("accept", "application/json").send().await;
        
         match res {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<OntimeRundown>().await {
                        Ok(rundown) => {
                            // Convert to simple event list using flat_order to preserve sequence
                            let mut events = Vec::new();
                            for id in rundown.flat_order {
                                if let Some(entry) = rundown.entries.get(&id) {
                                    events.push(OntimeEvent {
                                        id: entry.id.clone(),
                                        title: if entry.title.is_empty() { 
                                            format!("{} ({})", entry.entry_type, entry.id)
                                        } else {
                                            entry.title.clone() 
                                        },
                                        event_type: entry.entry_type.clone(),
                                    });
                                }
                            }
                            Ok(events)
                        },
                        Err(e) => Err(format!("Parse error: {}", e))
                    }
                } else { Err(format!("API Error: {}", response.status())) }
            },
            Err(_) => Err("Ontime Connection failed".to_string())
        }
    });

    // Fetch full Ontime rundown for timeline visualization
    let mut ontime_timeline_resource = use_resource(|| async move {
        let settings = AppSettings::load();
        let base_url = format!("http://{}:{}", settings.ot_host, settings.ot_port);
        let client = reqwest::Client::new();
        
        let url = format!("{}/data/rundowns/current", base_url);
        let res = client.get(&url).header("accept", "application/json").send().await;
        
        match res {
            Ok(response) => {
                if response.status().is_success() {
                    // Parse as Value first for flexibility
                    match response.json::<serde_json::Value>().await {
                        Ok(json) => {
                            // Extract fields manually
                            let id = json.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let title = json.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let flat_order: Vec<String> = json.get("flatOrder")
                                .and_then(|v| v.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_default();
                            let revision = json.get("revision").and_then(|v| v.as_u64()).unwrap_or(0);
                            
                            let mut entries_map = std::collections::HashMap::new();
                            if let Some(entries_obj) = json.get("entries").and_then(|v| v.as_object()) {
                                for (entry_id, entry_val) in entries_obj {
                                    let entry = OntimeEntry {
                                        id: entry_val.get("id").and_then(|v| v.as_str()).unwrap_or(entry_id).to_string(),
                                        entry_type: entry_val.get("type").and_then(|v| v.as_str()).unwrap_or("event").to_string(),
                                        title: entry_val.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        cue: entry_val.get("cue").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        note: entry_val.get("note").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        colour: entry_val.get("colour").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        duration: entry_val.get("duration").and_then(|v| v.as_u64()).unwrap_or(0),
                                        time_start: entry_val.get("timeStart").and_then(|v| v.as_u64()).unwrap_or(0),
                                        time_end: entry_val.get("timeEnd").and_then(|v| v.as_u64()).unwrap_or(0),
                                        parent: entry_val.get("parent").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                        _extra: std::collections::HashMap::new(),
                                    };
                                    entries_map.insert(entry_id.clone(), entry);
                                }
                            }
                            
                            Ok(OntimeRundown {
                                id,
                                title,
                                order: vec![],
                                flat_order,
                                entries: entries_map,
                                revision,
                            })
                        }
                        Err(e) => Err(format!("Parse error: {}", e))
                    }
                } else { Err(format!("API Error: {}", response.status())) }
            },
            Err(_) => Err("Ontime Connection failed".to_string())
        }
    });
    
    // Fetch playlist items when name changes
    let mut playlist_resource = use_resource(move || async move {
        let name = playlist_name();
        let settings = AppSettings::load();
        if name.is_empty() { return Ok(vec![]); }
        
        let url = format!("http://{}:{}/v1/playlist/{}", settings.pp_host, settings.pp_port, name);
        let client = reqwest::Client::new();
        let res = client.get(&url).header("accept", "application/json").send().await;
        
        match res {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<PlaylistResponse>().await {
                        Ok(data) => Ok(data.items),
                        Err(e) => Err(format!("Parse error: {}", e))
                    }
                } else { Err(format!("API Error: {}", response.status())) }
            },
            Err(_) => Err("Connection failed".to_string())
        }
    });
    
    // Helper to check if item is already added (in any standalone or group)
    let is_item_added = |item_id: &str| -> bool {
        formatter_items.read().iter().any(|item| match item {
            FormatterItem::Standalone(entry) => entry.item_id == item_id,
            FormatterItem::Group { entries, .. } => entries.iter().any(|e| e.item_id == item_id),
            FormatterItem::Reference { .. } => false,
        })
    };
    
    // Add item - standalone or to selected group
    let add_item = move |item: &PlaylistItem| {
        let entry = TimedEntry {
            item_id: item.id.uuid.clone(),
            name: item.id.name.clone(),
            item_type: item.item_type.clone(),
            duration: "00:05:00".to_string(),
            end_time: "00:00:00".to_string(),
            count_to_end: false,
            insertion_index: None,
        };
        
        let mut items = formatter_items.write();
        
        // Check if already added anywhere
        let already_added = items.iter().any(|fi| match fi {
            FormatterItem::Standalone(e) => e.item_id == item.id.uuid,
            FormatterItem::Group { entries, .. } => entries.iter().any(|e| e.item_id == item.id.uuid),
            FormatterItem::Reference { .. } => false,
        });
        
        if already_added {
            return;
        }
        
        match selected_group_idx() {
            Some(idx) if idx < items.len() => {
                // Add to selected group
                if let FormatterItem::Group { entries, .. } = &mut items[idx] {
                    entries.push(entry);
                }
            }
            _ => {
                // Add as standalone
                items.push(FormatterItem::Standalone(entry));
            }
        }
    };

    rsx! {
        div { class: "formatter-page",
            // Connection Status Header
            div { class: "connection-status-bar",
                div { class: "status-item",
                    if let Some(Ok(_)) = &*playlists_resource.read() {
                        div { class: "indicator-dot success" }
                        "ProPresenter: Connected"
                    } else {
                        div { class: "indicator-dot error" }
                        "ProPresenter: Disconnected"
                    }
                }
                div { class: "status-item",
                    if let Some(Ok(_)) = &*ontime_resource.read() {
                        div { class: "indicator-dot success" }
                        "Ontime: Connected"
                    } else {
                        div { class: "indicator-dot error" }
                        "Ontime: Disconnected"
                    }
                }
            }

            div { class: "formatter-container",
                // Left Panel - Playlist Source
                div { class: "formatter-panel source-panel",
                    div { class: "panel-title", "PLAYLIST SOURCE" }
                    div { class: "control-group",
                        label { "SELECT PLAYLIST" }
                        div { class: "input-row",
                            select {
                                class: "playlist-select",
                                onchange: move |e| {
                                    logs.write()
                                        .push(
                                            format!(
                                                "[{}] Selected playlist: {}",
                                                chrono::Local::now().format("%H:%M:%S"),
                                                e.value(),
                                            ),
                                        );
                                    playlist_name.set(e.value());
                                },
                                option { value: "", "-- Select Playlist --" }
                                match &*playlists_resource.read() {
                                    Some(Ok(playlists)) => rsx! {
                                        for pl in playlists {
                                            option { value: "{pl.id.name}", "{pl.id.name}" }
                                        }
                                    },
                                    Some(Err(_)) => rsx! {
                                        option { "Error loading..." }
                                    },
                                    None => rsx! {
                                        option { "Loading..." }
                                    },
                                }
                            }
                            button {
                                class: "btn-icon",
                                onclick: move |_| {
                                    logs.write()
                                        .push(
                                            format!(
                                                "[{}] Refreshing playlists...",
                                                chrono::Local::now().format("%H:%M:%S"),
                                            ),
                                        );
                                    playlist_resource.restart();
                                },
                                "↻"
                            }
                        }
                    }
                    div { class: "playlist-items",
                        match &*playlist_resource.read() {
                            Some(Ok(items)) => rsx! {
                                for item in items {
                                    {
                                        let item_clone = item.clone();
                                        let is_added = formatter_items
                                            .read()
                                            .iter()
                                            .any(|fi| match fi {
                                                FormatterItem::Standalone(e) => e.item_id == item.id.uuid,
                                                FormatterItem::Group { entries, .. } => entries.iter().any(|e| e.item_id == item.id.uuid),
                                                FormatterItem::Reference { .. } => false,
                                            });
                                        rsx! {
                                            div {
                                                class: if is_added { "source-item added" } else { "source-item" },
                                                onclick: move |_| {
                                                    let entry = TimedEntry {
                                                        item_id: item_clone.id.uuid.clone(),
                                                        name: item_clone.id.name.clone(),
                                                        item_type: item_clone.item_type.clone(),
                                                        duration: "00:05:00".to_string(),
                                                        end_time: "00:00:00".to_string(),
                                                        count_to_end: false,
                                                        insertion_index: None,
                                                    };
                                                    
                                                    let mut items = formatter_items.write();
                                                    
                                                    // Check if already added
                                                    let already_added = items.iter().any(|fi| match fi {
                                                        FormatterItem::Standalone(e) => e.item_id == item_clone.id.uuid,
                                                        FormatterItem::Group { entries, .. } => entries.iter().any(|e| e.item_id == item_clone.id.uuid),
                                                        FormatterItem::Reference { .. } => false,
                                                    });
                                                    
                                                    if already_added {
                                                        return;
                                                    }
                                                    
                                                    logs.write().push(format!("[{}] Added item: {}", chrono::Local::now().format("%H:%M:%S"), item_clone.id.name));
                                                    
                                                    match selected_group_idx() {
                                                        Some(idx) if idx < items.len() => {
                                                            if let FormatterItem::Group { entries, .. } = &mut items[idx] {
                                                                entries.push(entry);
                                                            }
                                                        }
                                                        _ => {
                                                            items.push(FormatterItem::Standalone(entry));
                                                        }
                                                    }
                                                },
                                                if is_added {
                                                    span { class: "added-badge", "✓" }
                                                }
                                                span { class: "item-index", "{item.id.index + 1}" }
                                                span { class: "item-name", "{item.id.name}" }
                                                span { class: "item-type", "{item.item_type}" }
                                            }
                                        }
                                    }
                                }
                            },
                            Some(Err(e)) => rsx! {
                                div { class: "error-msg", "⚠ {e}" }
                            },
                            None => rsx! {
                                div { class: "loading-msg", "Loading..." }
                            },
                        }
                    }
                }
                // Right Panel - Formatter Groups
                div { class: "formatter-panel groups-panel",
                    div { class: "panel-title", "ONTIME FORMATTER" }
                    // Insertion selector removed - replaced by Reference Items list logic
                    div { class: "insertion-selector",
                         div { style: "color: var(--text-muted); font-size: 0.8rem; padding: 0 0 10px 0;",
                             "Click items in Timeline to add insertion points."
                         }
                    }
                    // Timeline Panel - removed placeholder
                    button {
                        class: "btn-add-group",
                        onclick: move |_| {
                            let mut items = formatter_items.write();
                            let group_count = items.iter().filter(|i| matches!(i, FormatterItem::Group { .. })).count();
                            let new_id = format!("group-{}", group_count + 1);
                            items.push(FormatterItem::Group {
                                id: new_id,
                                name: format!("GROUP {}", group_count + 1),
                                color: "#779BE7".to_string(),
                                entries: vec![],
                                collapsed: false,
                            });
                            // Auto-select new group
                            selected_group_idx.set(Some(items.len() - 1));
                        },
                        "+ NEW GROUP"
                    }
                    div { class: "groups-list",
                        // Render all items (standalone and groups) in unified list
                        for (item_idx, formatter_item) in formatter_items.read().iter().enumerate() {
                            match formatter_item {
                                FormatterItem::Standalone(entry) => rsx! {
                                    div {
                                        class: "timeline-entry",
                                        div { class: "entry-main",
                                            span { class: "entry-title", "{entry.name}" }
                                            span { class: "entry-duration", "{entry.duration}" }
                                            button {
                                                class: "btn-remove",
                                                onclick: move |_| {
                                                    formatter_items.write().remove(item_idx);
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                },
                                FormatterItem::Group { id, name, color, entries, collapsed } => {
                                    let name_clone = name.clone();
                                    let color_clone = color.clone();
                                    let entry_count = entries.len();
                                    rsx! {
                                        div {
                                            class: if selected_group_idx() == Some(item_idx) { "group-card selected" } else { "group-card" },
                                            onclick: move |_| {
                                                // Toggle selection
                                                if selected_group_idx() == Some(item_idx) {
                                                    selected_group_idx.set(None);
                                                } else {
                                                    selected_group_idx.set(Some(item_idx));
                                                }
                                            },
                                            // Group Header
                                            div { class: "group-header",
                                                input {
                                                    class: "group-name-input",
                                                    value: "{name_clone}",
                                                    onclick: move |e| e.stop_propagation(),
                                                    oninput: move |e| {
                                                        if let FormatterItem::Group { name, .. } = &mut formatter_items.write()[item_idx] {
                                                            *name = e.value();
                                                        }
                                                    },
                                                }
                                                div { style: "width: 30px; height: 30px; border-radius: 4px; margin-right: 8px; background-color: {color_clone}; border: 1px solid rgba(255,255,255,0.3);" }
                                                input {
                                                    class: "group-color-input",
                                                    style: "width: 80px; padding: 4px 8px; border-radius: 4px; border: 1px solid rgba(255,255,255,0.2); background: rgba(0,0,0,0.3); color: white; font-family: monospace;",
                                                    value: "{color_clone}",
                                                    placeholder: "#779BE7",
                                                    onclick: move |e| e.stop_propagation(),
                                                    oninput: move |e| {
                                                        if let FormatterItem::Group { color, .. } = &mut formatter_items.write()[item_idx] {
                                                            *color = e.value();
                                                        }
                                                    },
                                                }
                                                span { class: "group-count", "{entry_count} items" }
                                                button {
                                                    class: "btn-remove",
                                                    title: "Delete Group",
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        formatter_items.write().remove(item_idx);
                                                        selected_group_idx.set(None);
                                                    },
                                                    "×"
                                                }
                                            }
                                            // Nested entries under group
                                            for (entry_idx, entry) in entries.iter().enumerate() {
                                                div { class: "timeline-entry nested",
                                                    div { class: "entry-main",
                                                        span { class: "entry-title", "{entry.name}" }
                                                        span { class: "entry-duration", "{entry.duration}" }
                                                        button {
                                                            class: "btn-remove",
                                                            onclick: move |e| {
                                                                e.stop_propagation();
                                                                if let FormatterItem::Group { entries, .. } = &mut formatter_items.write()[item_idx] {
                                                                    entries.remove(entry_idx);
                                                                }
                                                            },
                                                            "×"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                FormatterItem::Reference { title, item_type, mode, .. } => {
                                    rsx! {
                                        div { class: "reference-item",
                                            div { class: "ref-icon", 
                                                if matches!(mode, InsertionMode::Into) { "↳" } else { "↓" }
                                            }
                                            div { class: "ref-details",
                                                span { class: "ref-label", 
                                                    if matches!(mode, InsertionMode::Into) { "INSERT INTO" } else { "INSERT AFTER" }
                                                }
                                                span { class: "ref-title", "{title}" }
                                            }
                                            button {
                                                class: "btn-remove",
                                                onclick: move |_| {
                                                    formatter_items.write().remove(item_idx);
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Push to OnTime button
                    div { class: "action-bar",
                        button {
                            class: "btn-push",
                            onclick: move |_| {
                                logs.write()
                                    .push(
                                        format!(
                                            "[{}] Starting Push to Ontime...",
                                            chrono::Local::now().format("%H:%M:%S"),
                                        ),
                                    );
                                let items_data = formatter_items.read().clone();
                                let ontime_events = ontime_resource.read().clone();
                                spawn(async move {
                                    let settings = AppSettings::load();
                                    let base_url = format!("http://{}:{}", settings.ot_host, settings.ot_port);
                                    let client = reqwest::Client::new();
                                    
                                    // Fetch current rundown ID first
                                    let rundown_url = format!("{}/data/rundowns/current", base_url);
                                    let rundown_id = match client.get(&rundown_url).header("accept", "application/json").send().await {
                                        Ok(resp) if resp.status().is_success() => {
                                            resp.json::<serde_json::Value>().await.ok()
                                                .and_then(|j| j.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                                        }
                                        _ => None
                                    };
                                    
                                    let rundown_id = match rundown_id {
                                        Some(id) => id,
                                        None => {
                                            logs.write().push(format!("[{}] ✗ Could not get current rundown ID", chrono::Local::now().format("%H:%M:%S")));
                                            return;
                                        }
                                    };
                                    
                                    let endpoint = format!("{}/data/rundowns/{}/entry", base_url, rundown_id);
                                    logs.write().push(format!("[{}] Push to: {} ({})", chrono::Local::now().format("%H:%M:%S"), rundown_id, endpoint));
                                    
                                    let (existing_ids, existing_titles): (Vec<String>, Vec<String>) = match ontime_events {
                                        Some(Ok(events)) => (
                                            events.iter().map(|e| e.id.clone()).collect(),
                                            events.iter().map(|e| e.title.clone()).collect(),
                                        ),
                                        _ => (vec![], vec![]),
                                    };
                                    
                                    // Determine initial 'after' and 'parent' - default to END if no explicit ref is first
                                    let mut current_after_id = existing_ids.last().cloned();
                                    let mut current_parent_id: Option<String> = None;

                                    for item in items_data {
                                        match item {
                                            FormatterItem::Reference { id, item_type: _, mode, .. } => {
                                                // Update current insertion context
                                                match mode {
                                                    InsertionMode::After => {
                                                        current_after_id = Some(id.clone());
                                                        // When inserting AFTER an item, we need to know its parent to stay in same group
                                                        // We can try to look it up from 'existing_ids' if we had parent info,
                                                        // but simpler is to assume 'parent' matches the referenced item's parent? 
                                                        // Or just let Ontime handle it. If we set 'after=ID', Ontime places it after ID. 
                                                        // We should CLEAR current_parent_id to avoid forcing it if we can't look it up easily.
                                                        // Ideally we should lookup the item to see its parent.
                                                        // Since we don't have easy parent lookup map here, we might just nullify parent
                                                        // and rely on 'after'. (Ontime usually infers parent from 'after' sibling).
                                                        current_parent_id = None; 
                                                    }
                                                    InsertionMode::Into => {
                                                        current_parent_id = Some(id.clone());
                                                        // When inserting INTO a group, 'after' should be the last child.
                                                        // But we don't know the last child easily here.
                                                        // Sending just 'parent' with no 'after' usually appends to group.
                                                        current_after_id = None; 
                                                    }
                                                }
                                                logs.write().push(format!("[{}] Set Context: Mode {:?} ID {}", chrono::Local::now().format("%H:%M:%S"), mode, id));
                                            }
                                            FormatterItem::Standalone(entry) => {
                                                if existing_titles.contains(&entry.name) {
                                                    continue;
                                                }
                                                let duration_ms = parse_duration_to_ms(&entry.duration);
                                                let mut event_payload = serde_json::json!({
                                                    "type": "event",
                                                    "title": entry.name,
                                                    "duration": duration_ms
                                                });
                                                
                                                if let Some(ref pid) = current_parent_id {
                                                    event_payload["parent"] = serde_json::json!(pid);
                                                }
                                                
                                                if let Some(ref a) = current_after_id {
                                                    event_payload["after"] = serde_json::json!(a);
                                                }
                                                
                                                logs.write().push(format!("[{}] Event: {}", chrono::Local::now().format("%H:%M:%S"), entry.name));
                                                
                                                match client.post(&endpoint).header("Content-Type", "application/json").json(&event_payload).send().await {
                                                    Ok(resp) => {
                                                        let status = resp.status();
                                                        if status.is_success() {
                                                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                                                if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                                                                    current_after_id = Some(id.to_string());
                                                                    logs.write().push(format!("[{}] ✓ Created: {}", chrono::Local::now().format("%H:%M:%S"), id));
                                                                }
                                                            }
                                                        } else {
                                                            let body = resp.text().await.unwrap_or_default();
                                                            logs.write().push(format!("[{}] ✗ Error {}: {}", chrono::Local::now().format("%H:%M:%S"), status, body));
                                                        }
                                                    }
                                                    Err(e) => {
                                                        logs.write().push(format!("[{}] ✗ Request failed: {}", chrono::Local::now().format("%H:%M:%S"), e));
                                                    }
                                                }
                                            }
                                            FormatterItem::Group { name, color, entries, .. } => {
                                                // Create group first
                                                let mut group_payload = serde_json::json!({
                                                    "type": "group",
                                                    "title": name,
                                                    "colour": color
                                                });
                                                
                                                if let Some(ref pid) = current_parent_id {
                                                    group_payload["parent"] = serde_json::json!(pid);
                                                }
                                                
                                                if let Some(ref a) = current_after_id {
                                                    group_payload["after"] = serde_json::json!(a);
                                                }
                                                logs.write().push(format!("[{}] Group: {}", chrono::Local::now().format("%H:%M:%S"), name));
                                                
                                                let group_id = match client.post(&endpoint).header("Content-Type", "application/json").json(&group_payload).send().await {
                                                    Ok(resp) => {
                                                        let status = resp.status();
                                                        if status.is_success() {
                                                            let result = resp.json::<serde_json::Value>().await.ok()
                                                                .and_then(|j| j.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()));
                                                            if let Some(ref id) = result {
                                                                logs.write().push(format!("[{}] ✓ Group created: {}", chrono::Local::now().format("%H:%M:%S"), id));
                                                            }
                                                            result
                                                        } else {
                                                            let body = resp.text().await.unwrap_or_default();
                                                            logs.write().push(format!("[{}] ✗ Group error {}: {}", chrono::Local::now().format("%H:%M:%S"), status, body));
                                                            None
                                                        }
                                                    }
                                                    Err(e) => {
                                                        logs.write().push(format!("[{}] ✗ Group request failed: {}", chrono::Local::now().format("%H:%M:%S"), e));
                                                        None
                                                    }
                                                };
                                                
                                                if let Some(ref gid) = group_id {
                                                    current_after_id = Some(gid.clone());
                                                }
                                                
                                                // Add entries under this NEW group
                                                // NOTE: The internal entries of this new group will always have THIS group as parent.
                                                // Their 'after' logic restarts within the group.
                                                let mut internal_after_id = None; 
                                                
                                                for entry in entries {
                                                    if existing_titles.contains(&entry.name) {
                                                        logs.write().push(format!("[{}] Skipping duplicate: {}", chrono::Local::now().format("%H:%M:%S"), entry.name));
                                                        continue;
                                                    }
                                                    let duration_ms = parse_duration_to_ms(&entry.duration);
                                                    let mut event_payload = serde_json::json!({
                                                        "type": "event",
                                                        "title": entry.name,
                                                        "duration": duration_ms
                                                    });
                                                    if let Some(ref gid) = group_id {
                                                        event_payload["parent"] = serde_json::json!(gid);
                                                    }
                                                    
                                                    // For internal items, we chain them one after another inside the group
                                                    if let Some(ref a) = internal_after_id {
                                                        event_payload["after"] = serde_json::json!(a);
                                                    }
                                                    
                                                    logs.write().push(format!("[{}] Entry: {}", chrono::Local::now().format("%H:%M:%S"), entry.name));
                                                    
                                                    match client.post(&endpoint).header("Content-Type", "application/json").json(&event_payload).send().await {
                                                        Ok(resp) => {
                                                            let status = resp.status();
                                                            if status.is_success() {
                                                                if let Ok(json) = resp.json::<serde_json::Value>().await {
                                                                    if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                                                                        internal_after_id = Some(id.to_string());
                                                                        logs.write().push(format!("[{}] ✓ Created: {}", chrono::Local::now().format("%H:%M:%S"), id));
                                                                    }
                                                                }
                                                            } else {
                                                                let body = resp.text().await.unwrap_or_default();
                                                                logs.write().push(format!("[{}] ✗ Error {}: {}", chrono::Local::now().format("%H:%M:%S"), status, body));
                                                            }
                                                        }
                                                        Err(e) => {
                                                            logs.write().push(format!("[{}] ✗ Request failed: {}", chrono::Local::now().format("%H:%M:%S"), e));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    
                                    // REFRESH TIMELINE AFTER PUSH
                                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                    logs.write().push(format!("[{}] Push Complete - refreshing timeline...", chrono::Local::now().format("%H:%M:%S")));
                                    
                                    // Clear items and selection BUT keep context selection if user wants?
                                    // User requested: "everything should be reseted to default after pushing"
                                    // Clear items and selection
                                    formatter_items.write().clear();
                                    selected_group_idx.set(None);
                                    
                                    ontime_timeline_resource.restart();
                                    ontime_resource.restart();
                                });
                            },
                            "PUSH TO ONTIME"
                        }
                    }
                }
                // RIGHT PANEL - ONTIME TIMELINE
                div { class: "formatter-panel timeline-panel",
                    div { class: "panel-title",
                        span { "ONTIME TIMELINE" }
                        button {
                            class: "btn-icon small",
                            onclick: move |_| ontime_timeline_resource.restart(),
                            "↻"
                        }
                    }
                    div { class: "timeline-content",
                        match &*ontime_timeline_resource.read() {
                            Some(Ok(rundown)) => rsx! {
                                div { class: "rundown-info",
                                    span { class: "rundown-name", "{rundown.title}" }
                                    span { class: "rundown-meta", "Rev: {rundown.revision}" }
                                }
                                div { class: "timeline-list",
                                    for entry_id in &rundown.flat_order {
                                        if let Some(entry) = rundown.entries.get(entry_id) {
                                            {
                                                let entry_id_clone = entry.id.clone();
                                                let entry_title_clone = entry.title.clone();
                                                let entry_type = entry.entry_type.clone();
                                                
                                                // Calculate which entries are referenced to show feedback in timeline
                                                let is_referenced = formatter_items.read().iter().any(|item| matches!(item, FormatterItem::Reference { id, .. } if *id == entry_id_clone));

                                                let base_class = if entry.parent.is_some() {
                                                    "timeline-entry nested selectable"
                                                } else if entry.entry_type == "group" {
                                                    "timeline-entry group selectable"
                                                } else {
                                                    "timeline-entry selectable"
                                                };
                                                
                                                rsx! {
                                                    div {
                                                        class: if is_referenced { format!("{} selected", base_class) } else { base_class.to_string() },
                                                        style: if !entry.colour.is_empty() { format!("border-left-color: {}", entry.colour) } else { "".to_string() },
                                                        onclick: move |_| {
                                                            let mode = if entry_type == "group" { InsertionMode::Into } else { InsertionMode::After };
                                                            logs.write().push(format!("[{}] Added Reference: {}", chrono::Local::now().format("%H:%M:%S"), entry_title_clone));
                                                            
                                                            // Add Reference to the end of the list
                                                            formatter_items.write().push(FormatterItem::Reference {
                                                                id: entry_id_clone.clone(),
                                                                title: entry_title_clone.clone(),
                                                                item_type: entry_type.clone(),
                                                                mode,
                                                            });
                                                        },
                                                        
                                                        div { class: "entry-main",
                                                            if !entry.cue.is_empty() {
                                                                span { class: "entry-cue", "{entry.cue}" }
                                                            }
                                                            span { class: "entry-title", "{entry.title}" }
                                                            if entry.duration > 0 {
                                                                span { class: "entry-duration",
                                                                    "{format_ms_to_duration(entry.duration)}"
                                                                }
                                                            }
                                                            
                                                            // Check icon if referenced
                                                            if is_referenced {
                                                                div { style: "margin-left: auto; color: var(--accent-ot); font-weight: bold;", "✓ REF" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            Some(Err(e)) => rsx! {
                                div { class: "error-msg", "⚠ {e}" }
                            },
                            None => rsx! {
                                div { class: "loading-msg", "Loading..." }
                            },
                        }
                    }
                }
            }
            // Live Logs Panel
            div { class: "console-panel",
                div { class: "panel-header", "LIVE LOGS" }
                div { class: "console-output",
                    for log in logs.read().iter() {
                        div { class: "log-entry", "{log}" }
                    }
                }
            }
        }
    }
}
