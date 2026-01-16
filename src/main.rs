use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/settings")]
    Settings {},
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
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "true" }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;700&display=swap"
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
                    to: Route::Settings {},
                    class: "nav-item",
                    active_class: "active",
                    "SETTINGS"
                }
            }
        }
        div { id: "app-container",
            Outlet::<Route> {}
        }
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
                                    oninput: move |e| playlist_name.set(e.value())
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
                                }
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
                        if is_syncing() { "STOP BRIDGE" } else { "START BRIDGE" }
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
                         div { class: "log-entry success", "[Success] Fetched {items.len()} items from playlist" }
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
                            oninput: move |e| settings.write().pp_host = e.value()
                        }
                    }
                    div { class: "input-group",
                        label { "Port" }
                        input {
                            value: "{settings.read().pp_port}",
                            oninput: move |e| settings.write().pp_port = e.value()
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
                            oninput: move |e| settings.write().ot_host = e.value()
                        }
                    }
                    div { class: "input-group",
                        label { "Port" }
                        input {
                            value: "{settings.read().ot_port}",
                            oninput: move |e| settings.write().ot_port = e.value()
                        }
                    }
                }

                // Sync Settings
                div { class: "settings-card full-width",
                    div { class: "card-header", "SYNC SETTINGS" }
                    div { class: "setting-row",
                        label { "Sync Interval (ms)" }
                        input { type: "number", value: "500" }
                    }
                    div { class: "setting-row",
                        label { "Auto-start on launch" }
                        input { type: "checkbox" }
                    }
                }

                div { class: "settings-card full-width",
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
                    div { 
                        style: "text-align: center; color: var(--color-success); margin-top: 10px; height: 20px;",
                        "{save_status}" 
                    }
                }
            }
        }
    }
}
