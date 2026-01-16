
#[component]
fn TimePicker(
    value: String,
    on_close: EventHandler<()>,
    on_save: EventHandler<String>,
) -> Element {
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

    rsx! {
        div { class: "time-picker-overlay", onclick: move |_| on_close.call(()),
            div { class: "time-picker-modal", onclick: move |e| e.stop_propagation(),
                div { class: "tp-header", "Set Time" }
                div { class: "tp-inputs",
                    div { class: "tp-column",
                        label { "HR" }
                        input {
                            r#type: "number",
                            min: "0",
                            max: "23",
                            class: "tp-input",
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
                            class: "tp-input",
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
                            class: "tp-input",
                            value: "{s}",
                            oninput: move |e| s.set(e.value().parse().unwrap_or(0))
                        }
                    }
                }
                div { class: "tp-actions",
                    button { class: "btn-secondary", onclick: move |_| on_close.call(()), "Cancel" }
                    button { class: "btn-primary", onclick: save, "Save" }
                }
            }
        }
    }
}
