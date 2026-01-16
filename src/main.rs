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
    let mut groups = use_signal(|| vec![
        FormatterGroup {
            id: "group-1".to_string(),
            name: "group".to_string(),
            color: "#779BE7".to_string(),
            entries: vec![],
            collapsed: false,
        }
    ]);
    let mut insertion_point = use_signal(|| "End".to_string());
    let mut selected_group = use_signal(|| 0usize);
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
                    match response.json::<OntimeRundown>().await {
                        Ok(rundown) => Ok(rundown),
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
    
    // Helper to check if item is already in any group
    let is_item_added = |item_id: &str| -> bool {
        groups.read().iter().any(|g| g.entries.iter().any(|e| e.item_id == item_id))
    };
    
    // Add item to selected group
    let add_item_to_group = move |item: &PlaylistItem| {
        let idx = selected_group();
        let mut g = groups.write();
        if idx < g.len() {
            // Check if already added
            if !g[idx].entries.iter().any(|e| e.item_id == item.id.uuid) {
                g[idx].entries.push(TimedEntry {
                    item_id: item.id.uuid.clone(),
                    name: item.id.name.clone(),
                    item_type: item.item_type.clone(),
                    duration: "00:05:00".to_string(),
                    end_time: "00:00:00".to_string(),
                    count_to_end: false,
                    insertion_index: None,
                });
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
                                        let is_added = groups
                                            .read()
                                            .iter()
                                            .any(|g| g.entries.iter().any(|e| e.item_id == item.id.uuid));
                                        rsx! {
                                            div {
                                                class: if is_added { "source-item added" } else { "source-item" },
                                                onclick: move |_| {
                                                    let idx = selected_group();
                                                    let mut g = groups.write();
                                                    if idx < g.len()
                                                        && !g[idx].entries.iter().any(|e| e.item_id == item_clone.id.uuid)
                                                    {
                                                        logs.write()
                                                            .push(
                                                                format!(
                                                                    "[{}] Added item: {}",
                                                                    chrono::Local::now().format("%H:%M:%S"),
                                                                    item_clone.id.name,
                                                                ),
                                                            );
                                                        g[idx]
                                                            .entries
                                                            .push(TimedEntry {
                                                                item_id: item_clone.id.uuid.clone(),
                                                                name: item_clone.id.name.clone(),
                                                                item_type: item_clone.item_type.clone(),
                                                                duration: "00:05:00".to_string(),
                                                                end_time: "00:00:00".to_string(),
                                                                count_to_end: false,
                                                                insertion_index: None,
                                                            });
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
                    // Insertion point selector
                    div { class: "insertion-selector",
                        label { "Insert New Items After:" }
                        div { class: "input-row",
                            select {
                                class: "insertion-select",
                                value: "{insertion_point}",
                                onchange: move |e| insertion_point.set(e.value()),
                                option { value: "End", "End (Append to Bottom)" }
                                match &*ontime_resource.read() {
                                    Some(Ok(events)) => rsx! {
                                        for event in events {
                                            option { value: "{event.id}", "After: {event.title}" }
                                        }
                                    },
                                    _ => rsx! {},
                                }
                            }
                            button {
                                class: "btn-icon",
                                onclick: move |_| ontime_resource.restart(),
                                "↻"
                            }
                        }
                    }
                    // Timeline Panel
                    div { class: "timeline-panel",
                        div { class: "panel-title", "TIMELINE" }
                        div { class: "timeline-content",
                            // This is where the timeline visualization would go
                            "Timeline visualization coming soon!"
                        }
                    }
                    button {
                        class: "btn-add-group",
                        onclick: move |_| {
                            let mut g = groups.write();
                            let next_num = g.len() + 1;
                            let new_id = format!("group-{}", next_num);
                            g.push(FormatterGroup {
                                id: new_id,
                                name: format!("GROUP {}", next_num),
                                color: "#779BE7".to_string(),
                                entries: vec![],
                                collapsed: false,
                            });
                        },
                        "+ NEW GROUP"
                    }
                    div { class: "groups-list",
                        for (group_idx , group) in groups.read().iter().enumerate() {
                            div {
                                class: if selected_group() == group_idx { "group-card selected" } else { "group-card" },
                                onclick: move |_| selected_group.set(group_idx),
                                // Group Header
                                div { class: "group-header",
                                    input {
                                        class: "group-name-input",
                                        value: "{group.name}",
                                        onclick: move |e| e.stop_propagation(),
                                        oninput: move |e| {
                                            groups.write()[group_idx].name = e.value();
                                        },
                                    }
                                    // Color preview box showing the current color
                                    div { style: "width: 30px; height: 30px; border-radius: 4px; margin-right: 8px; background-color: {group.color}; border: 1px solid rgba(255,255,255,0.3);" }
                                    // Text input for hex color
                                    input {
                                        class: "group-color-input",
                                        style: "width: 80px; padding: 4px 8px; border-radius: 4px; border: 1px solid rgba(255,255,255,0.2); background: rgba(0,0,0,0.3); color: white; font-family: monospace;",
                                        value: "{group.color}",
                                        placeholder: "#779BE7",
                                        onclick: move |e| e.stop_propagation(),
                                        oninput: move |e| {
                                            groups.write()[group_idx].color = e.value();
                                        },
                                    }
                                    span { class: "group-count", "{group.entries.len()} items" }
                                    button {
                                        class: "btn-remove",
                                        title: "Delete Group",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            let name = groups.read()[group_idx].name.clone();
                                            groups.write().remove(group_idx);
                                            // Adjust selection if needed
                                            if selected_group() >= groups.read().len() && !groups.read().is_empty() {
                                                selected_group.set(groups.read().len() - 1);
                                            } else if groups.read().is_empty() {
                                                selected_group.set(0); 
                                            }
                                            logs.write().push(format!("[{}] Removed group: {}", chrono::Local::now().format("%H:%M:%S"), name));
                                        },
                                        "×"
                                    }
                                }
                                // Entries Table
                                if !group.entries.is_empty() {
                                    div { class: "entries-table",
                                        div { class: "entries-header",
                                            span { "NAME" }
                                            span { "DURATION" }
                                            span { "END TIME" }
                                            span { "COUNT" }
                                            span { "" }
                                        }
                                        for (entry_idx , entry) in group.entries.iter().enumerate() {
                                            div { class: "entry-row",
                                                span { class: "entry-name", "{entry.name}" }
                                                input {
                                                    class: "entry-input",
                                                    value: "{entry.duration}",
                                                                                                // Note: In a full impl, would handle oninput
                                                }
                                                input {
                                                    class: "entry-input",
                                                    value: "{entry.end_time}",
                                                }
                                                input {
                                                    r#type: "checkbox",
                                                    checked: entry.count_to_end,
                                                }
                                                button {
                                                    class: "btn-remove",
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        let mut g = groups.write();
                                                        if group_idx < g.len() {
                                                            g[group_idx].entries.remove(entry_idx);
                                                        }
                                                    },
                                                    "×"
                                                }
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
                                let groups_data = groups.read().clone();
                                let insertion_choice = insertion_point();
                                let ontime_events = ontime_resource.read().clone();
                                spawn(async move {
                                    let settings = AppSettings::load();
                                    let base_url = format!("http://{}:{}", settings.ot_host, settings.ot_port);
                                    let client = reqwest::Client::new();
                                    let rundown_id = "default";
                                    let endpoint = format!("{}/data/rundowns/{}/entry", base_url, rundown_id);
                                    logs.write()
                                        .push(
                                            format!(
                                                "[{}] Push URL: {}",
                                                chrono::Local::now().format("%H:%M:%S"),
                                                endpoint,
                                            ),
                                        );
                                    let (existing_ids, existing_titles): (Vec<String>, Vec<String>) = match ontime_events {
                                        Some(Ok(events)) => {
                                            (
                                                events.iter().map(|e| e.id.clone()).collect(),
                                                events.iter().map(|e| e.title.clone()).collect(),
                                            )
                                        }
                                        _ => (vec![], vec![]),
                                    };
                                    let mut current_after_id = if insertion_choice == "Start" {
                                        None
                                    } else if insertion_choice == "End" {
                                        existing_ids.last().cloned()
                                    } else {
                                        if existing_ids.contains(&insertion_choice) {
                                            Some(insertion_choice.clone())
                                        } else {
                                            existing_ids.last().cloned()
                                        }
                                    };
                                    println!(
                                        "Insertion choice: {}, current_after_id: {:?}",
                                        insertion_choice,
                                        current_after_id,
                                    );
                                    for group in groups_data {
                                        println!("Creating group: {} with color: {}", group.name, group.color);
                                        let group_payload = serde_json::json!(
                                            { "type" : "group", "title" : group.name, "colour" : group.color }
                                        );
                                        let mut final_group_payload = group_payload.clone();
                                        if let Some(ref a) = current_after_id {
                                            final_group_payload["after"] = serde_json::json!(a);
                                        }
                                        println!("Group payload: {}", final_group_payload);
                                        logs.write()
                                            .push(
                                                format!(
                                                    "[{}] Group payload: {}",
                                                    chrono::Local::now().format("%H:%M:%S"),
                                                    final_group_payload,
                                                ),
                                            );
                                        let group_res = client
                                            .post(&endpoint)
                                            .header("Content-Type", "application/json")
                                            .json(&final_group_payload)
                                            .send()
                                            .await;
                                        let group_id = match group_res {
                                            Ok(resp) => {
                                                if resp.status().is_success() {
                                                    match resp.json::<serde_json::Value>().await {
                                                        Ok(json) => {
                                                            json.get("id")
                                                                .and_then(|v| v.as_str())
                                                                .map(|s| s.to_string())
                                                        }
                                                        Err(_) => None,
                                                    }
                                                } else {
                                                    None
                                                }
                                            }
                                            Err(_) => None,
                                        };
                                        if let Some(ref gid) = group_id {
                                            current_after_id = Some(gid.clone());
                                        }
                                        for entry in group.entries {
                                            if existing_titles.contains(&entry.name) {
                                                println!("Skipping duplicate item: {}", entry.name);
                                                continue;
                                            }
                                            let duration_ms = parse_duration_to_ms(&entry.duration);
                                            let mut event_payload = serde_json::json!(
                                                { "type" : "event", "title" : entry.name, "duration" :
                                                duration_ms }
                                            );
                                            if let Some(ref gid) = group_id {
                                                event_payload["parent"] = serde_json::json!(gid);
                                            }
                                            if let Some(ref a) = current_after_id {
                                                event_payload["after"] = serde_json::json!(a);
                                            }
                                            logs.write()
                                                .push(
                                                    format!(
                                                        "[{}] Event payload: {}",
                                                        chrono::Local::now().format("%H:%M:%S"),
                                                        event_payload,
                                                    ),
                                                );
                                            let event_res = client
                                                .post(&endpoint)
                                                .header("Content-Type", "application/json")
                                                .json(&event_payload)
                                                .send()
                                                .await;
                                                
                                            // Update after ID for next item
                                            if let Ok(resp) = event_res {
                                                if resp.status().is_success() {
                                                     if let Ok(json) = resp.json::<serde_json::Value>().await {
                                                        if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                                                            current_after_id = Some(id.to_string());
                                                        }
                                                     }
                                                }
                                            }
                                        }
                                    }
                                    
                                    // REFRESH TIMELINE AFTER PUSH
                                    // We can't directly call restart() from here easily without cloning the signal,
                                    // but usually we would want to trigger a refresh.
                                    // For now, user can click refresh.
                                    logs.write().push(format!("[{}] Push Complete", chrono::Local::now().format("%H:%M:%S")));
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
                                            div {
                                                class: if entry.parent.is_some() { "timeline-entry nested" } else { 
                                                    if entry.entry_type == "group" { "timeline-entry group" } else { "timeline-entry" }
                                                },
                                                style: if !entry.colour.is_empty() { format!("border-left-color: {}", entry.colour) } else { "".to_string() },
                                                
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
