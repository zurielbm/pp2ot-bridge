use dioxus::prelude::*;
use crate::types::AppSettings;

/// Settings page
#[component]
pub fn Settings() -> Element {
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

                div { class: "settings-card full-width",
                    div { class: "card-header", "DEFAULTS" }
                    div { class: "input-group",
                        label { "Default Duration" }
                        input {
                            value: "{settings.read().default_duration}",
                            oninput: move |e| settings.write().default_duration = e.value(),
                        }
                    }
                    div { class: "input-group",
                        label { "Default End Time" }
                        input {
                            value: "{settings.read().default_end_time}",
                            oninput: move |e| settings.write().default_end_time = e.value(),
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
                                Err(_e) => save_status.set("Failed to save!"),
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
