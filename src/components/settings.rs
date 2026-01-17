use dioxus::prelude::*;
use crate::types::AppSettings;

/// Settings page
#[component]
pub fn Settings() -> Element {
    // Initialize with loaded settings
    let mut settings = use_signal(AppSettings::load);
    let mut save_status = use_signal(|| "");

    rsx! {
        div { class: "p-8 max-w-5xl mx-auto flex flex-col gap-6 font-mono text-white bg-zinc-950 min-h-screen",
            div { class: "flex justify-between items-end mb-2",
                h1 { class: "text-2xl font-extrabold tracking-wider text-zinc-100 uppercase", "CONFIGURATION" }
            }

            div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                // ProPresenter Config
                div { class: "bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden flex flex-col",
                    div { class: "p-4 flex items-center gap-3 border-b border-zinc-800 bg-zinc-900/50",
                        div { class: "w-2 h-2 rounded-full bg-zinc-600" }
                        span { class: "text-xs font-bold text-zinc-500 uppercase tracking-widest", "PROPRESENTER SOURCE" }
                    }
                    div { class: "p-6 flex flex-col gap-6",
                        div {
                            label { class: "text-[0.65rem] font-bold text-zinc-500 uppercase tracking-wider mb-2 block", "HOST ADDRESS" }
                            input {
                                class: "w-full bg-zinc-950 border border-zinc-800 rounded p-3 text-sm font-mono text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-all placeholder-zinc-700",
                                value: "{settings.read().pp_host}",
                                oninput: move |e| settings.write().pp_host = e.value(),
                            }
                        }
                        div {
                            label { class: "text-[0.65rem] font-bold text-zinc-500 uppercase tracking-wider mb-2 block", "PORT" }
                            input {
                                class: "w-full bg-zinc-950 border border-zinc-800 rounded p-3 text-sm font-mono text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-all placeholder-zinc-700",
                                value: "{settings.read().pp_port}",
                                oninput: move |e| settings.write().pp_port = e.value(),
                            }
                        }
                    }
                }

                // OnTime Config
                div { class: "bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden flex flex-col",
                    div { class: "p-4 flex items-center gap-3 border-b border-zinc-800 bg-zinc-900/50",
                        div { class: "w-2 h-2 rounded-full bg-zinc-600" }
                        span { class: "text-xs font-bold text-zinc-500 uppercase tracking-widest", "ONTIME DESTINATION" }
                    }
                    div { class: "p-6 flex flex-col gap-6",
                        div {
                            label { class: "text-[0.65rem] font-bold text-zinc-500 uppercase tracking-wider mb-2 block", "HOST ADDRESS" }
                            input {
                                class: "w-full bg-zinc-950 border border-zinc-800 rounded p-3 text-sm font-mono text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-all placeholder-zinc-700",
                                value: "{settings.read().ot_host}",
                                oninput: move |e| settings.write().ot_host = e.value(),
                            }
                        }
                        div {
                            label { class: "text-[0.65rem] font-bold text-zinc-500 uppercase tracking-wider mb-2 block", "PORT" }
                            input {
                                class: "w-full bg-zinc-950 border border-zinc-800 rounded p-3 text-sm font-mono text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-all placeholder-zinc-700",
                                value: "{settings.read().ot_port}",
                                oninput: move |e| settings.write().ot_port = e.value(),
                            }
                        }
                    }
                }
            }

            // Defaults
            div { class: "bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden flex flex-col",
                div { class: "p-4 flex items-center gap-3 border-b border-zinc-800 bg-zinc-900/50",
                    div { class: "w-2 h-2 rounded-full bg-zinc-600" }
                    span { class: "text-xs font-bold text-zinc-500 uppercase tracking-widest", "DEFAULTS" }
                }
                div { class: "p-6 flex flex-col gap-6",
                    div {
                        label { class: "text-[0.65rem] font-bold text-zinc-500 uppercase tracking-wider mb-2 block", "DEFAULT DURATION" }
                        input {
                            class: "w-full bg-zinc-950 border border-zinc-800 rounded p-3 text-sm font-mono text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-all placeholder-zinc-700",
                            value: "{settings.read().default_duration}",
                            oninput: move |e| settings.write().default_duration = e.value(),
                        }
                    }
                    div {
                        label { class: "text-[0.65rem] font-bold text-zinc-500 uppercase tracking-wider mb-2 block", "DEFAULT END TIME" }
                        input {
                            class: "w-full bg-zinc-950 border border-zinc-800 rounded p-3 text-sm font-mono text-zinc-200 focus:outline-none focus:border-emerald-500/50 transition-all placeholder-zinc-700",
                            value: "{settings.read().default_end_time}",
                            oninput: move |e| settings.write().default_end_time = e.value(),
                        }
                    }
                }
            }

            // Sync Settings
            div { class: "bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden flex flex-col",
                div { class: "p-4 flex items-center gap-3 border-b border-zinc-800 bg-zinc-900/50",
                    div { class: "w-2 h-2 rounded-full bg-zinc-600" }
                    span { class: "text-xs font-bold text-zinc-500 uppercase tracking-widest", "SYNC SETTINGS" }
                }
                div { class: "flex flex-col",
                    div { class: "flex items-center justify-between p-4 px-6 border-b border-zinc-800/50",
                        label { class: "text-sm font-mono text-zinc-300", "Sync Interval (ms)" }
                        input { 
                            r#type: "number", 
                            class: "bg-transparent text-right font-mono text-zinc-200 w-20 focus:outline-none",
                            value: "500" 
                        }
                    }
                    div { class: "flex items-center justify-between p-4 px-6",
                        label { class: "text-sm font-mono text-zinc-300", "Auto-start on launch" }
                        input { 
                            r#type: "checkbox",
                            class: "w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-emerald-500 focus:ring-0 focus:ring-offset-0 cursor-pointer"
                        }
                    }
                }
            }

            button {
                class: "w-full py-3.5 bg-emerald-500 hover:bg-emerald-400 text-zinc-950 font-extrabold rounded shadow-lg transition-all tracking-wider text-sm mt-4 uppercase",
                onclick: move |_| {
                    match settings.read().save() {
                        Ok(_) => save_status.set("Configuration Saved!"),
                        Err(_e) => save_status.set("Failed to save!"),
                    }
                },
                "SAVE CONFIGURATION"
            }
            
            if !save_status.read().is_empty() {
                div { class: "text-center text-emerald-500 font-bold font-mono text-xs animate-fade-in",
                    "{save_status}"
                }
            }
        }
    }
}
