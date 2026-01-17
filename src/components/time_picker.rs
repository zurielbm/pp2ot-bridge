use dioxus::prelude::*;
use crate::types::{AppSettings, TimeField};

#[component]
pub fn TimePicker(
    value: String,
    field: TimeField,
    on_close: EventHandler<()>,
    on_save: EventHandler<String>,
) -> Element {
    let mut settings = use_signal(AppSettings::load);
    // Parse initial value "HH:MM:SS"
    let (initial_h, initial_m, initial_s) = {
        let parts: Vec<&str> = value.split(':').collect();
        let h = parts.get(0).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        let m = parts.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        let s = parts.get(2).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        (h, m, s)
    };

    let mut h = use_signal(|| initial_h);
    let mut m = use_signal(|| initial_m);
    let mut s = use_signal(|| initial_s);

    let save = move |_| {
        let new_time = format!("{:02}:{:02}:{:02}", h(), m(), s());
        on_save.call(new_time);
    };

    let add_favorite = move |_| {
        let new_time = format!("{:02}:{:02}:{:02}", h(), m(), s());
        let mut current_settings = settings.write();
        match field {
            TimeField::Duration => {
                if !current_settings.favorite_durations.contains(&new_time) {
                    current_settings.favorite_durations.push(new_time);
                    let _ = current_settings.save();
                }
            }
            TimeField::EndTime => {
                if !current_settings.favorite_end_times.contains(&new_time) {
                    current_settings.favorite_end_times.push(new_time);
                    let _ = current_settings.save();
                }
            }
        }
    };
    
    // Validation Helpers
    let is_valid_h = h() >= 0 && h() <= 23;
    let is_valid_m = m() >= 0 && m() <= 59;
    let is_valid_s = s() >= 0 && s() <= 59;
    let is_form_valid = is_valid_h && is_valid_m && is_valid_s;

    let favorites = match field {
        TimeField::Duration => settings.read().favorite_durations.clone(),
        TimeField::EndTime => settings.read().favorite_end_times.clone(),
    };

    rsx! {
        div { class: "time-picker-overlay", onclick: move |_| on_close.call(()),
            div { class: "time-picker-modal", onclick: move |e| e.stop_propagation(),
                div { class: "tp-header",
                    if matches!(field, TimeField::EndTime) {
                        span { "Set End Time" }
                        div { class: "tp-subtitle", "(24-hour format)" }
                    } else {
                        span { "Set Duration" }
                    }
                }
                div { class: "tp-inputs",
                    div { class: "tp-column",
                        label { "HR" }
                        input {
                            r#type: "number",
                            min: "0",
                            max: "23",
                            class: if is_valid_h { "tp-input" } else { "tp-input invalid" },
                            value: "{h}",
                            oninput: move |e| h.set(e.value().parse().unwrap_or(0))
                        }
                    }
                    span { class: "tp-separator", ":" }
                    div { class: "tp-column",
                        label { "MIN" }
                        input {
                            r#type: "number",
                            min: "0",
                            max: "59",
                            class: if is_valid_m { "tp-input" } else { "tp-input invalid" },
                            value: "{m}",
                            oninput: move |e| m.set(e.value().parse().unwrap_or(0))
                        }
                    }
                    span { class: "tp-separator", ":" }
                    div { class: "tp-column",
                        label { "SEC" }
                        input {
                            r#type: "number",
                            min: "0",
                            max: "59",
                            class: if is_valid_s { "tp-input" } else { "tp-input invalid" },
                            value: "{s}",
                            oninput: move |e| s.set(e.value().parse().unwrap_or(0))
                        }
                    }
                }
                
                if !is_form_valid {
                    div { class: "tp-error-msg",
                        "Invalid Time Format. Please ensure 0-23 hours and 0-59 minutes/seconds."
                    }
                }
                
                if !favorites.is_empty() {
                    div { class: "tp-favorites",
                        div { class: "tp-fav-label", "FAVORITES" }
                        div { class: "tp-fav-list",
                            for fav in favorites {
                                div { class: "tp-fav-item",
                                    {
                                        let fav_click = fav.clone();
                                        let fav_display = fav.clone();
                                        rsx! {
                                            button {
                                                class: "tp-fav-chip",
                                                onclick: move |_| {
                                                    let parts: Vec<&str> = fav_click.split(':').collect();
                                                    if parts.len() == 3 {
                                                        h.set(parts[0].parse().unwrap_or(0));
                                                        m.set(parts[1].parse().unwrap_or(0));
                                                        s.set(parts[2].parse().unwrap_or(0));
                                                    }
                                                },
                                                "{fav_display}"
                                            }
                                        }
                                    }
                                    {
                                        let fav_del = fav.clone();
                                        let mut settings_del = settings;
                                        let field_del = field;
                                        rsx! {
                                            button {
                                                class: "tp-fav-delete",
                                                onclick: move |e: Event<MouseData>| {
                                                    e.stop_propagation();
                                                    let mut current_settings = settings_del.write();
                                                    match field_del {
                                                        TimeField::Duration => {
                                                            current_settings.favorite_durations.retain(|x| x != &fav_del);
                                                            let _ = current_settings.save();
                                                        }
                                                        TimeField::EndTime => {
                                                            current_settings.favorite_end_times.retain(|x| x != &fav_del);
                                                            let _ = current_settings.save();
                                                        }
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

                div { class: "tp-actions",
                    button { class: "btn-secondary", onclick: move |_| on_close.call(()), "Cancel" }
                    button { 
                        class: "btn-secondary",
                        disabled: !is_form_valid, 
                        onclick: add_favorite, 
                        "♥ Add" 
                    }
                    button { 
                        class: "btn-primary",
                        disabled: !is_form_valid,
                        onclick: save, 
                        "Save" 
                    }
                }
            }
        }
    }
}
