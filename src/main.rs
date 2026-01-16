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

/// Home page - Bridge Control Center
#[component]
fn Home() -> Element {
    let mut is_syncing = use_signal(|| false);

    rsx! {
        div { class: "page-container",
            div { class: "status-grid",
                // ProPresenter Status Card
                div { class: "status-card pro-presenter",
                    div { class: "card-header", "PROPRESENTER" }
                    div { class: "card-body",
                        div { class: "status-indicator online",
                            span { class: "indicator-dot" }
                            "CONNECTED"
                        }
                        div { class: "status-details",
                            div { "Running on: localhost:1025" }
                            div { "Last heartbeat: 2s ago" }
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
                    div { class: "card-header", "ONTIME" }
                    div { class: "card-body",
                        div { class: "status-indicator online",
                            span { class: "indicator-dot" }
                            "CONNECTED"
                        }
                        div { class: "status-details",
                            div { "Running on: localhost:4001" }
                            div { "Last heartbeat: 2s ago" }
                        }
                    }
                }
            }

            // Live Log Console
            div { class: "console-panel",
                div { class: "panel-header", "LIVE LOGS" }
                div { class: "console-output",
                    div { class: "log-entry info", "[10:21:02] Application started" }
                    div { class: "log-entry success", "[10:21:03] Connected to ProPresenter API" }
                    div { class: "log-entry success", "[10:21:03] Connected to OnTime API" }
                    div { class: "log-entry system", "[10:21:05] Bridge ready for sync" }
                }
            }
        }
    }
}

/// Settings page
#[component]
fn Settings() -> Element {
    let mut pp_host = use_signal(|| "localhost".to_string());
    let mut pp_port = use_signal(|| "1025".to_string());
    let mut ot_host = use_signal(|| "localhost".to_string());
    let mut ot_port = use_signal(|| "4001".to_string());

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
                            value: "{pp_host}",
                            oninput: move |e| pp_host.set(e.value())
                        }
                    }
                    div { class: "input-group",
                        label { "Port" }
                        input {
                            value: "{pp_port}",
                            oninput: move |e| pp_port.set(e.value())
                        }
                    }
                    button { class: "btn-secondary", "TEST CONNECTION" }
                }

                // OnTime Config
                div { class: "settings-card",
                    div { class: "card-header", "ONTIME DESTINATION" }
                    div { class: "input-group",
                        label { "Host Address" }
                        input {
                            value: "{ot_host}",
                            oninput: move |e| ot_host.set(e.value())
                        }
                    }
                    div { class: "input-group",
                        label { "Port" }
                        input {
                            value: "{ot_port}",
                            oninput: move |e| ot_port.set(e.value())
                        }
                    }
                    button { class: "btn-secondary", "TEST CONNECTION" }
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
            }
        }
    }
}
