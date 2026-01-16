use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/settings")]
    Settings {},
    #[route("/formatter")]
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
                    to: Route::Home {},
                    class: "nav-item",
                    active_class: "active",
                    "DASHBOARD"
                }
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

#[component]
fn Home() -> Element {
    let mut is_syncing = use_signal(|| false);
    let mut playlist_name = use_signal(|| "Sunday SPANISH 10am".to_string());
    
    // Resource to fetch playlist items
    let mut playlist_resource = use_resource(move || async move {
        let name = playlist_name();
        // Load settings to get the correct host/port
        // Note: In a real async environment, we might want to pass these as props or context
        // to avoid reading file on every request, but for this button click it's fine.
        let settings = AppSettings::load(); 
        
        // Don't fetch if empty to avoid initial error
        if name.is_empty() {
            return Ok(vec![]);
        }
        
        let url = format!("http://{}:{}/v1/playlist/{}", settings.pp_host, settings.pp_port, name);
        // We'll return a result to handle errors gracefully
        let client = reqwest::Client::new();
        let res = client.get(&url)
            .header("accept", "application/json")
            .send()
            .await;
            
        match res {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<PlaylistResponse>().await {
                        Ok(data) => Ok(data.items),
                        Err(e) => Err(format!("Failed to parse data: {}", e))
                    }
                } else {
                    Err(format!("API Error: {}", response.status()))
                }
            },
            Err(_) => Err("Failed to connect to ProPresenter".to_string())
        }
    });

    rsx! {
        div { class: "page-container",
            div { class: "status-grid",
                // ProPresenter Status Card
                div { class: "status-card pro-presenter",
                    div { class: "card-header", "PROPRESENTER SOURCE" }
                    div { class: "card-body",
                        div { class: "status-indicator online",
                            span { class: "indicator-dot" }
                            "CONNECTED"
                        }
                        div { class: "control-group",
                            label { "PLAYLIST NAME" }
                            div { class: "input-row",
                                input {
                                    value: "{playlist_name}",
                                    oninput: move |e| playlist_name.set(e.value()),
                                }
                                button {
                                    class: "btn-icon",
                                    onclick: move |_| playlist_resource.restart(),
                                    "↻"
                                }
                            }
                        }

                        // Playlist Items List
                        div { class: "playlist-preview",
                            match &*playlist_resource.read() {
                                Some(Ok(items)) => rsx! {
                                    div { class: "items-header",
                                        span { "ITEMS ({items.len()})" }
                                    }
                                    div { class: "items-list",
                                        for item in items {
                                            div { class: "playlist-item",
                                                span { class: "item-index", "{item.id.index + 1}" }
                                                span { class: "item-name", "{item.id.name}" }
                                                span {
                                                    class: "item-type",
                                                    style: "font-size: 0.65rem; color: var(--text-muted); margin-left: auto; text-transform: uppercase;",
                                                    "{item.item_type}"
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

                // Sync Control Center
                div { class: "control-center",
                    div { class: "sync-status",
                        if is_syncing() {
                            div { class: "sync-active", "SYNC ACTIVE" }
                        } else {
                            div { class: "sync-inactive", "SYNC PAUSED" }
                        }
                    }
                    button {
                        class: if is_syncing() { "btn-sync stop" } else { "btn-sync start" },
                        onclick: move |_| is_syncing.set(!is_syncing()),
                        if is_syncing() {
                            "STOP BRIDGE"
                        } else {
                            "START BRIDGE"
                        }
                    }
                }

                // OnTime Status Card
                div { class: "status-card on-time",
                    div { class: "card-header", "ONTIME DESTINATION" }
                    div { class: "card-body",
                        div { class: "status-indicator online",
                            span { class: "indicator-dot" }
                            "CONNECTED"
                        }
                        div { class: "status-details",
                            // Display loaded settings or defaults if we wanted,
                            // but for now static text is okay or we could fetch them too.
                            // Let's keep it simple for this step.
                            div { "Running on configured port" }
                            div { "Last heartbeat: 2s ago" }
                            div { "Next API call in: 500ms" }
                        }
                    }
                }
            }

            // Live Log Console
            div { class: "console-panel",
                div { class: "panel-header", "LIVE LOGS" }
                div { class: "console-output",
                    div { class: "log-entry info", "[10:21:02] Application started" }
                    if let Some(Err(e)) = &*playlist_resource.read() {
                        div { class: "log-entry error", "[Error] {e}" }
                    }
                    if let Some(Ok(items)) = &*playlist_resource.read() {
                        div { class: "log-entry success",
                            "[Success] Fetched {items.len()} items from playlist"
                        }
                    }
                }
            }
        }
    }
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
    // Signal for selected insertion point (global for simplicity)
    let mut insertion_point = use_signal(|| "End".to_string());
    let mut selected_group = use_signal(|| 0usize);
    
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
        
        // Fetch from default rundown
        let url = format!("{}/data/rundowns/default/entries", base_url);
        let res = client.get(&url).header("accept", "application/json").send().await;
        
         match res {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Vec<OntimeEvent>>().await {
                        Ok(events) => Ok(events),
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
                            onclick: move |_| playlist_resource.restart(),
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
                            option { value: "Start", "Start (Top of List)" }
                            match &*ontime_resource.read() {
                                Some(Ok(events)) => rsx! {
                                    for event in events {
                                        option { value: "{event.id}", "{event.title}" }
                                    }
                                    if !events.is_empty() {
                                        option { value: "End", "End (Bottom of List)" }
                                    }
                                },
                                _ => rsx! {
                                    option { value: "End", "End" }
                                },
                            }
                        }
                        button {
                            class: "btn-icon",
                            onclick: move |_| ontime_resource.restart(),
                            "↻"
                        }
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
                                input {
                                    r#type: "color",
                                    class: "group-color-input",
                                    style: "width: 40px; height: 30px; padding: 0; border: none; background: none; cursor: pointer;",
                                    value: "{group.color}",
                                    onclick: move |e| e.stop_propagation(),
                                    oninput: move |e| {
                                        groups.write()[group_idx].color = e.value();
                                    },
                                }
                                span { class: "group-count", "{group.entries.len()} items" }
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
                button {
                    class: "btn-push",
                    onclick: move |_| {
                        let groups_data = groups.read().clone();
                        let insertion_choice = insertion_point();
                        let ontime_events = ontime_resource.read().clone(); // Capture current ontime events
                        spawn(async move {
                            let settings = AppSettings::load();
                            let base_url = format!("http://{}:{}", settings.ot_host, settings.ot_port);
                            let client = reqwest::Client::new();

                            // Use "default" rundown for now - could be made configurable
                            let rundown_id = "default";
                            let endpoint = format!("{}/data/rundowns/{}/entry", base_url, rundown_id);

                            // Fetch existing entries from Ontime directly to verify duplicates
                            let (existing_ids, existing_titles): (Vec<String>, Vec<String>) = match ontime_events {

                                // Determine initial AFTER ID based on user selection
                                // If the user selected a specific ID, verify it exists, otherwise fallback to end

                                // 1. Create the group header via POST

                                // Determine after for the group itself
                                // Note: We only set 'after' for the group if we are inserting into a flow

                                // Update current_after_id to be the group we just created

                                // Skip if already exists by checking TITLE

                                // NESTING: Items are nested by being placed sequentially AFTER the group (or after the previous item).
                                // The 'after' field tells Ontime to insert this event after the specified ID.

                                // Update current_after_id so the NEXT item chains after THIS one
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
                            for group in groups_data {
                                let group_payload = serde_json::json!(
                                    { "type" : "group", "title" : group.name, "colour" : group.color }
                                );
                                let mut final_group_payload = group_payload.clone();
                                if let Some(ref a) = current_after_id {
                                    final_group_payload["after"] = serde_json::json!(a);
                                }
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
                                    
                                    // NESTING FIX: Use "parent" field to put item INSIDE the group
                                    if let Some(ref gid) = group_id {
                                        event_payload["parent"] = serde_json::json!(gid);
                                    }
                                    
                                    // ORDERING: Use "after" field to place item after the previous one
                                    if let Some(ref a) = current_after_id {
                                        event_payload["after"] = serde_json::json!(a);
                                    }
                                    let event_res = client
                                        .post(&endpoint)
                                        .header("Content-Type", "application/json")
                                        .json(&event_payload)
                                        .send()
                                        .await;
                                    match event_res {
                                        Ok(resp) => {
                                            if resp.status().is_success() {
                                                match resp.json::<serde_json::Value>().await {
                                                    Ok(json) => {
                                                        let id = json
                                                            .get("id")
                                                            .and_then(|v| v.as_str())
                                                            .map(|s| s.to_string());
                                                        current_after_id = id;
                                                    }
                                                    Err(_) => {}
                                                }
                                            }
                                        }
                                        Err(_) => {}
                                    }
                                }
                            }
                            println!("Push to OnTime completed!");
                        });
                    },
                    "PUSH TO ONTIME"
                }
            }
        }
    }
}
