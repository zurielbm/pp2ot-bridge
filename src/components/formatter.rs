use dioxus::prelude::*;
use std::collections::HashMap;
use crate::types::{
    AppSettings, PlaylistResponse, PlaylistItem, PlaylistInfo,
    OntimeEvent, OntimeEntry, OntimeRundown,
    TimedEntry, FormatterItem, TimeEditContext, TimeField, InsertionMode,
};
use crate::utils::format_ms_to_duration;
use crate::utils::parse_duration_to_ms;
use crate::components::TimePicker;

#[component]
pub fn Formatter() -> Element {
    let mut playlist_name = use_signal(|| String::new());
    // Unified list of items (standalone entries and groups)
    let mut formatter_items = use_signal(|| Vec::<FormatterItem>::new());
    // None = standalone mode (append to end), Some(idx) = add inside group at that index
    let mut selected_group_idx = use_signal(|| Option::<usize>::None);
    let mut active_time_edit = use_signal(|| Option::<TimeEditContext>::None);
    let mut dragged_item_idx = use_signal(|| Option::<usize>::None);
    let mut drag_over_idx = use_signal(|| Option::<usize>::None);
    let mut logs = use_signal(|| vec![
        format!("[{}] System Ready", chrono::Local::now().format("%H:%M:%S"))
    ]);

    let _add_log = move |msg: String| {
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
                            
                            let mut entries_map = HashMap::new();
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
                                        _extra: HashMap::new(),
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
    let _is_item_added = |item_id: &str| -> bool {
        formatter_items.read().iter().any(|item| match item {
            FormatterItem::Standalone(entry) => entry.item_id == item_id,
            FormatterItem::Group { entries, .. } => entries.iter().any(|e| e.item_id == item_id),
            FormatterItem::Reference { .. } => false,
        })
    };
    
    // Add item - standalone or to selected group
    let _add_item = move |item: &PlaylistItem| {
        let entry = TimedEntry {
            item_id: item.id.uuid.clone(),
            name: item.id.name.clone(),
            item_type: item.item_type.clone(),
            duration: "00:05:00".to_string(),
            end_time: "00:00:00".to_string(),
            count_to_end: false,
            link_start: true,
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
                                                FormatterItem::Group { entries, .. } => {
                                                    entries.iter().any(|e| e.item_id == item.id.uuid)
                                                }
                                                FormatterItem::Reference { .. } => false,
                                            });
                                        rsx! {
                                            div {
                                                class: if is_added { "source-item added" } else { "source-item" },
                                                onclick: move |_| {
                                                    let suggested_end_time = {
                                                        let items_read = formatter_items.read();
                                                        let last_ref = items_read
                                                            .iter()
                                                            .rev()
                                                            .find_map(|item| {
                                                                if let FormatterItem::Reference { time_end, .. } = item {
                                                                    Some(*time_end)
                                                                } else {
                                                                    None
                                                                }
                                                            });
                                                        if let Some(ref_end) = last_ref {
                                                            if ref_end > 0 {
                                                                let settings = AppSettings::load();
                                                                let default_duration_ms = parse_duration_to_ms(
                                                                    &settings.default_duration,
                                                                );
                                                                format_ms_to_duration(ref_end + default_duration_ms)
                                                            } else {
                                                                "00:00:00".to_string()
                                                            }
                                                        } else {
                                                            "00:00:00".to_string()
                                                        }
                                                    };
                                                    let entry = TimedEntry {
                                                        item_id: item_clone.id.uuid.clone(),
                                                        name: item_clone.id.name.clone(),
                                                        item_type: item_clone.item_type.clone(),
                                                        duration: {
                                                            match AppSettings::load().default_duration.as_str() {
                                                                "" => "00:05:00".to_string(),
                                                                s => s.to_string(),
                                                            }
                                                        },
        
                
        
                                                        end_time: { suggested_end_time },
                                                        count_to_end: false,
                                                        link_start: true,
                                                        insertion_index: None,
                                                    };
                                                    let mut items = formatter_items.write();
                                                    let already_added = items
                                                        .iter()
                                                        .any(|fi| match fi {
                                                            FormatterItem::Standalone(e) => e.item_id == item_clone.id.uuid,
                                                            FormatterItem::Group { entries, .. } => {
                                                                entries.iter().any(|e| e.item_id == item_clone.id.uuid)
                                                            }
                                                            FormatterItem::Reference { .. } => false,
                                                        });
                

                                                    if already_added {
                                                        return;
                                                    }
                                                    logs.write()
                                                        .push(
                                                            format!(
                                                                "[{}] Added item: {}",
                                                                chrono::Local::now().format("%H:%M:%S"),
                                                                item_clone.id.name,
                                                            ),
                                                        );
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
                            let group_count = items
                                // Auto-select new group
                                .iter()
                                .filter(|i| matches!(i, FormatterItem::Group { .. }))
                                .count();
                            let new_id = format!("group-{}", group_count + 1);
                            items
                                .push(FormatterItem::Group {
                                    id: new_id,
                                    name: format!("GROUP {}", group_count + 1),
                                    color: "#779BE7".to_string(),
                                    entries: vec![],
                                    collapsed: false,
                                });
                            selected_group_idx.set(Some(items.len() - 1));
                        },
                        "+ NEW GROUP"
                    }
                    div { class: "groups-list",
                        // Render all items (standalone and groups) in unified list
                        for (item_idx, formatter_item) in formatter_items.read().iter().enumerate() {
                            {
                                // Drag handlers
                                // We need to capture the current item_idx for the closures
                                let current_idx = item_idx;
                                
                                // Determine if this item is being dragged to add styling class
                                let is_dragging = *dragged_item_idx.read() == Some(current_idx);
                                
                                match formatter_item {
                                FormatterItem::Standalone(entry) => {
                                    let duration_clone = entry.duration.clone();
                                    let end_time_clone = entry.end_time.clone();
                                    let count_to_end = entry.count_to_end;
                                    let link_start = entry.link_start;
                                    // Calculate drag over class
                                    let drag_over_class = if let (Some(dragged_idx), Some(over_idx)) = (*dragged_item_idx.read(), *drag_over_idx.read()) {
                                        if over_idx == current_idx && dragged_idx != current_idx {
                                            if dragged_idx < current_idx {
                                                " drag-over-bottom"
                                            } else {
                                                " drag-over-top"
                                            }
                                        } else {
                                            ""
                                        }
                                    } else {
                                        ""
                                    };

                                    rsx! {
                                        div { 
                                            class: if is_dragging { 
                                                format!("timeline-entry editable-entry draggable dragging{}", drag_over_class)
                                            } else { 
                                                format!("timeline-entry editable-entry draggable{}", drag_over_class)
                                            },
                                            draggable: true,
                                            ondragstart: move |_| {
                                                dragged_item_idx.set(Some(current_idx));
                                            },
                                            ondragenter: move |e| {
                                                e.prevent_default();
                                                // Only set if we are dragging something
                                                if dragged_item_idx.read().is_some() {
                                                    drag_over_idx.set(Some(current_idx));
                                                }
                                            },
                                            ondragover: move |e| {
                                                e.prevent_default();
                                            },
                                            ondrop: move |e| {
                                                e.prevent_default();
                                                let source_idx_opt = *dragged_item_idx.read();
                                                if let Some(source_idx) = source_idx_opt {
                                                    if source_idx != current_idx {
                                                        let mut items = formatter_items.write();
                                                        let item = items.remove(source_idx);
                                                        items.insert(current_idx, item);
                                                    }
                                                }
                                                dragged_item_idx.set(None);
                                                drag_over_idx.set(None);
                                            },
                                            div { class: "entry-main",
                                                span { class: "drag-handle", "⋮⋮" }
                                                span { class: "entry-title", "{entry.name}" }
                                                button {
                                                    class: "btn-remove",
                                                    onclick: move |_| {
                                                        formatter_items.write().remove(item_idx);
                                                    },
                                                    "×"
                                                }
                                            }
                                            div { class: "entry-fields",
                                                div { class: "field-group",
                                                    label { "Duration" }
                                                    input {
                                                        r#type: "text",
                                                        class: "time-input cursor-pointer",
                                                        value: "{duration_clone}",
                                                        readonly: true,
                                                        onclick: move |_| {
                                                            active_time_edit
                                                                .set(
                                                                    Some(TimeEditContext {
                                                                        item_idx,
                                                                        sub_item_idx: None,
                                                                        field: TimeField::Duration,
                                                                        current_value: duration_clone.clone(),
                                                                    }),
                                                                );
                                                        },
                                                    }
                                                }
                                                div { class: "field-group",
                                                    label { "End Time" }
                                                    input {
                                                        r#type: "text",
                                                        class: "time-input cursor-pointer",
                                                        value: "{end_time_clone}",
                                                        readonly: true,
                                                        onclick: move |_| {
                                                            active_time_edit
                                                                .set(
                                                                    Some(TimeEditContext {
                                                                        item_idx,
                                                                        sub_item_idx: None,
                                                                        field: TimeField::EndTime,
                                                                        current_value: end_time_clone.clone(),
                                                                    }),
                                                                );
                                                        },
                                                    }
                                                }
                                                div { class: "field-group checkbox-group",
                                                    input {
                                                        r#type: "checkbox",
                                                        id: "cte-{item_idx}",
                                                        checked: count_to_end,
                                                        onchange: move |e| {
                                                            if let FormatterItem::Standalone(ref mut ent) = &mut formatter_items
                                                                .write()[item_idx]
                                                            {
                                                                ent.count_to_end = e.checked();
                                                            }
                                                        },
                                                    }
                                                    label { r#for: "cte-{item_idx}", "CTE" }
                                                }
                                                div { class: "field-group checkbox-group",
                                                    input {
                                                        r#type: "checkbox",
                                                        id: "ls-{item_idx}",
                                                        checked: link_start,
                                                        onchange: move |e| {
                                                            if let FormatterItem::Standalone(ref mut ent) = &mut formatter_items
                                                                .write()[item_idx]
                                                            {
                                                                ent.link_start = e.checked();
                                                            }
                                                        },
                                                    }
                                                    label { r#for: "ls-{item_idx}", "Link" }
                                                }
                                            }
                                        }
                                    }
                                }
                                FormatterItem::Group { id: _, name, color, entries, collapsed: _ } => {
                                    let name_clone = name.clone();
                                    let color_clone = color.clone();
                                    let entry_count = entries.len();
                                    // Calculate drag over class
                                    let drag_over_class = if let (Some(dragged_idx), Some(over_idx)) = (*dragged_item_idx.read(), *drag_over_idx.read()) {
                                        if over_idx == current_idx && dragged_idx != current_idx {
                                            if dragged_idx < current_idx {
                                                " drag-over-bottom"
                                            } else {
                                                " drag-over-top"
                                            }
                                        } else {
                                            ""
                                        }
                                    } else {
                                        ""
                                    };

                                    rsx! {
                                        div {
                                            class: if selected_group_idx() == Some(item_idx) { 
                                                format!("group-card selected draggable{}", drag_over_class)
                                            } else { 
                                                format!("group-card draggable{}", drag_over_class)
                                            },
                                            draggable: true,
                                            ondragstart: move |_| {
                                                dragged_item_idx.set(Some(current_idx));
                                            },
                                            ondragenter: move |e| {
                                                e.prevent_default();
                                                if dragged_item_idx.read().is_some() {
                                                    drag_over_idx.set(Some(current_idx));
                                                }
                                            },
                                            ondragover: move |e| {
                                                e.prevent_default();
                                            },
                                            ondrop: move |e| {
                                                e.prevent_default();
                                                let source_idx_opt = *dragged_item_idx.read();
                                                if let Some(source_idx) = source_idx_opt {
                                                    if source_idx != current_idx {
                                                        let mut items = formatter_items.write();
                                                        let item = items.remove(source_idx);
                                                        items.insert(current_idx, item);
                                                        
                                                        // Update selection if we moved the selected group
                                                        if selected_group_idx() == Some(source_idx) {
                                                            selected_group_idx.set(Some(current_idx));
                                                        } else if selected_group_idx() == Some(current_idx) {
                                                            selected_group_idx.set(None);
                                                        }
                                                    }
                                                }
                                                dragged_item_idx.set(None);
                                                drag_over_idx.set(None);
                                            },
                                            onclick: move |_| {
                                                if selected_group_idx() == Some(item_idx) {
                                                    selected_group_idx.set(None);
                                                } else {
                                                    selected_group_idx.set(Some(item_idx));
                                                }
                                            },
                                            div { class: "group-header",
                                                span { class: "drag-handle", "⋮⋮" }
                                                input {
                                                    class: "group-name-input",
                                                    value: "{name_clone}",
                                                    onclick: move |e| e.stop_propagation(),
                                                    oninput: move |e| {
                                                        if let FormatterItem::Group { name, .. } = &mut formatter_items.write()[item_idx]
                                                        {
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
                                                        if let FormatterItem::Group { color, .. } = &mut formatter_items
                                                            .write()[item_idx]
                                                        {
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
                                            for (entry_idx , entry) in entries.iter().enumerate() {
                                                {
                                                    let duration_clone = entry.duration.clone();
                                                    let end_time_clone = entry.end_time.clone();
                                                    let count_to_end = entry.count_to_end;
                                                    rsx! {
                                                        div { class: "timeline-entry nested editable-entry",
                                                            div { class: "entry-main",
                                                                span { class: "entry-title", "{entry.name}" }
                                                                button {
                                                                    class: "btn-remove",
                                                                    onclick: move |e| {
                                                                        e.stop_propagation();
                                                                        if let FormatterItem::Group { entries, .. } = &mut formatter_items
                                                                            .write()[item_idx]
                                                                        {
                                                                            entries.remove(entry_idx);
                                                                        }
                                                                    },
                                                                    "×"
                                                                }
                                                            }
                                                            div { class: "entry-fields",
                                                                div { class: "field-group",
                                                                    label { "Dur" }
                                                                    input {
                                                                        r#type: "text",
                                                                        class: "time-input cursor-pointer",
                                                                        value: "{duration_clone}",
                                                                        readonly: true,
                                                                        onclick: move |e| {
                                                                            e.stop_propagation();
                                                                            active_time_edit
                                                                                .set(
                                                                                    Some(TimeEditContext {
                                                                                        item_idx,
                                                                                        sub_item_idx: Some(entry_idx),
                                                                                        field: TimeField::Duration,
                                                                                        current_value: duration_clone.clone(),
                                                                                    }),
                                                                                );
                                                                        },
                                                                    }
                                                                }
                                                                div { class: "field-group",
                                                                    label { "End" }
                                                                    input {
                                                                        r#type: "text",
                                                                        class: "time-input cursor-pointer",
                                                                        value: "{end_time_clone}",
                                                                        readonly: true,
                                                                        onclick: move |e| {
                                                                            e.stop_propagation();
                                                                            active_time_edit
                                                                                .set(
                                                                                    Some(TimeEditContext {
                                                                                        item_idx,
                                                                                        sub_item_idx: Some(entry_idx),
                                                                                        field: TimeField::EndTime,
                                                                                        current_value: end_time_clone.clone(),
                                                                                    }),
                                                                                );
                                                                        },
                                                                    }
                                                                }
                                                                div { class: "field-group checkbox-group",
                                                                    input {
                                                                        r#type: "checkbox",
                                                                        id: "cte-{item_idx}-{entry_idx}",
                                                                        checked: count_to_end,
                                                                        onclick: move |e| e.stop_propagation(),
                                                                        onchange: move |e| {
                                                                            if let FormatterItem::Group { entries, .. } = &mut formatter_items
                                                                                .write()[item_idx]
                                                                            {
                                                                                entries[entry_idx].count_to_end = e.checked();
                                                                            }
                                                                        },
                                                                    }
                                                                    label { r#for: "cte-{item_idx}-{entry_idx}", "CTE" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                FormatterItem::Reference { title, item_type: _, mode, .. } => {
                                    // Calculate drag over class
                                    let drag_over_class = if let (Some(dragged_idx), Some(over_idx)) = (*dragged_item_idx.read(), *drag_over_idx.read()) {
                                        if over_idx == current_idx && dragged_idx != current_idx {
                                            if dragged_idx < current_idx {
                                                " drag-over-bottom"
                                            } else {
                                                " drag-over-top"
                                            }
                                        } else {
                                            ""
                                        }
                                    } else {
                                        ""
                                    };

                                    rsx! {
                                        div { 
                                            class: format!("reference-item draggable{}", drag_over_class),
                                            draggable: true,
                                            ondragstart: move |_| {
                                                dragged_item_idx.set(Some(current_idx));
                                            },
                                            ondragenter: move |e| {
                                                e.prevent_default();
                                                if dragged_item_idx.read().is_some() {
                                                    drag_over_idx.set(Some(current_idx));
                                                }
                                            },
                                            ondragover: move |e| {
                                                e.prevent_default();
                                            },
                                            ondrop: move |e| {
                                                e.prevent_default();
                                                let source_idx_opt = *dragged_item_idx.read();
                                                if let Some(source_idx) = source_idx_opt {
                                                    if source_idx != current_idx {
                                                        let mut items = formatter_items.write();
                                                        let item = items.remove(source_idx);
                                                        items.insert(current_idx, item);
                                                    }
                                                }
                                                dragged_item_idx.set(None);
                                                drag_over_idx.set(None);
                                            },
                                            div { class: "ref-icon",
                                                if matches!(mode, InsertionMode::Into) {
                                                    "↳"
                                                } else {
                                                    "↓"
                                                }
                                            }
                                            div { class: "ref-details",
                                                span { class: "drag-handle", "⋮⋮" }
                                                span { class: "ref-label",
                                                    if matches!(mode, InsertionMode::Into) {
                                                        "INSERT INTO"
                                                    } else {
                                                        "INSERT AFTER"
                                                    }
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
                                    let rundown_id = match client
                                        .get(&rundown_url)
                                        .header("accept", "application/json")
                                        .send()
                                        .await
                                    {
                                        Ok(resp) if resp.status().is_success() => {
                                            resp.json::<serde_json::Value>()
                                                .await
                                                .ok()
                                                .and_then(|j| {
                                                    j.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
                                                })
                                        }
                                        _ => None,
                                    };
                                    let rundown_id = match rundown_id {
                                        Some(id) => id,
                                        None => {
                                            logs.write()
                                                .push(
                                                    format!(
                                                        "[{}] ✗ Could not get current rundown ID",
                                                        chrono::Local::now().format("%H:%M:%S"),
                                                    ),
                                                );
                                            return;
                                        }
                                    };
                                    let endpoint = format!("{}/data/rundowns/{}/entry", base_url, rundown_id);
                                    logs.write()
                                        .push(
                                            format!(
                                                "[{}] Push to: {} ({})",
                                                chrono::Local::now().format("%H:%M:%S"),
                                                rundown_id,
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
                                    let mut current_after_id = existing_ids.last().cloned();
                                    let mut current_parent_id: Option<String> = None;
                                    for item in items_data {
                                        match item {
                                            FormatterItem::Reference { id, item_type: _, mode, .. } => {
                                                match mode {
                                                    InsertionMode::After => {
                                                        current_after_id = Some(id.clone());
                                                        current_parent_id = None;
                                                    }
                                                    InsertionMode::Into => {
                                                        current_parent_id = Some(id.clone());
                                                        current_after_id = None;
                                                    }
                                                }
                                                logs.write()
                                                    .push(
                                                        format!(
                                                            "[{}] Set Context: Mode {:?} ID {}",
                                                            chrono::Local::now().format("%H:%M:%S"),
                                                            mode,
                                                            id,
                                                        ),
                                                    );
                                            }
                                            FormatterItem::Standalone(entry) => {
                                                if existing_titles.contains(&entry.name) {
                                                    continue;
                                                }
                                                let duration_ms = parse_duration_to_ms(&entry.duration);
                                                let end_time_ms = parse_duration_to_ms(&entry.end_time);
                                                let mut event_payload = serde_json::json!(
                                                    { "type" : "event", "title" : entry.name, "duration" :
                                                    duration_ms, "timeEnd" : end_time_ms, "countToEnd" : entry
                                                    .count_to_end, "linkStart" : entry.link_start }
                                                );
                                                if let Some(ref pid) = current_parent_id {
                                                    event_payload["parent"] = serde_json::json!(pid);
                                                }
                                                if let Some(ref a) = current_after_id {
                                                    event_payload["after"] = serde_json::json!(a);
                                                }
                                                logs.write()
                                                    .push(
                                                        format!(
                                                            "[{}] Event: {}",
                                                            chrono::Local::now().format("%H:%M:%S"),
                                                            entry.name,
                                                        ),
                                                    );
                                                match client
                                                    .post(&endpoint)
                                                    .header("Content-Type", "application/json")
                                                    .json(&event_payload)
                                                    .send()
                                                    .await
                                                {
                                                    Ok(resp) => {
                                                        let status = resp.status();
                                                        if status.is_success() {
                                                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                                                if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                                                                    current_after_id = Some(id.to_string());
                                                                    logs.write()
                                                                        .push(
                                                                            format!(
                                                                                "[{}] ✓ Created: {}",
                                                                                chrono::Local::now().format("%H:%M:%S"),
                                                                                id,
                                                                            ),
                                                                        );
                                                                }
                                                            }
                                                        } else {
                                                            let body = resp.text().await.unwrap_or_default();
                                                            logs.write()
                                                                .push(
                                                                    format!(
                                                                        "[{}] ✗ Error {}: {}",
                                                                        chrono::Local::now().format("%H:%M:%S"),
                                                                        status,
                                                                        body,
                                                                    ),
                                                                );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        logs.write()
                                                            .push(
                                                                format!(
                                                                    "[{}] ✗ Request failed: {}",
                                                                    chrono::Local::now().format("%H:%M:%S"),
                                                                    e,
                                                                ),
                                                            );
                                                    }
                                                }
                                            }
                                            FormatterItem::Group { name, color, entries, .. } => {
                                                let mut group_payload = serde_json::json!(
                                                    { "type" : "group", "title" : name, "colour" : color }
                                                );
                                                if let Some(ref pid) = current_parent_id {
                                                    group_payload["parent"] = serde_json::json!(pid);
                                                }
                                                if let Some(ref a) = current_after_id {
                                                    group_payload["after"] = serde_json::json!(a);
                                                }
                                                logs.write()
                                                    .push(
                                                        format!(
                                                            "[{}] Group: {}",
                                                            chrono::Local::now().format("%H:%M:%S"),
                                                            name,
                                                        ),
                                                    );
                                                let group_id = match client
                                                    .post(&endpoint)
                                                    .header("Content-Type", "application/json")
                                                    .json(&group_payload)
                                                    .send()
                                                    .await
                                                {
                                                    Ok(resp) => {
                                                        let status = resp.status();
                                                        if status.is_success() {
                                                            let result = resp
                                                                .json::<serde_json::Value>()
                                                                .await
                                                                .ok()
                                                                .and_then(|j| {
                                                                    j.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
                                                                });
                                                            if let Some(ref id) = result {
                                                                logs.write()
                                                                    .push(
                                                                        format!(
                                                                            "[{}] ✓ Group created: {}",
                                                                            chrono::Local::now().format("%H:%M:%S"),
                                                                            id,
                                                                        ),
                                                                    );
                                                            }
                                                            result
                                                        } else {
                                                            let body = resp.text().await.unwrap_or_default();
                                                            logs.write()
                                                                .push(
                                                                    format!(
                                                                        "[{}] ✗ Group error {}: {}",
                                                                        chrono::Local::now().format("%H:%M:%S"),
                                                                        status,
                                                                        body,
                                                                    ),
                                                                );
                                                            None
                                                        }
                                                    }
                                                    Err(e) => {
                                                        logs.write()
                                                            .push(
                                                                format!(
                                                                    "[{}] ✗ Group request failed: {}",
                                                                    chrono::Local::now().format("%H:%M:%S"),
                                                                    e,
                                                                ),
                                                            );
                                                        None
                                                    }
                                                };
                                                if let Some(ref gid) = group_id {
                                                    current_after_id = Some(gid.clone());
                                                }
                                                let mut internal_after_id = None;
                                                for entry in entries {
                                                    if existing_titles.contains(&entry.name) {
                                                        logs.write()
                                                            .push(
                                                                format!(
                                                                    "[{}] Skipping duplicate: {}",
                                                                    chrono::Local::now().format("%H:%M:%S"),
                                                                    entry.name,
                                                                ),
                                                            );
                                                        continue;
                                                    }
                                                    let duration_ms = parse_duration_to_ms(&entry.duration);
                                                    let end_time_ms = parse_duration_to_ms(&entry.end_time);
                                                    let mut event_payload = serde_json::json!(
                                                        { "type" : "event", "title" : entry.name, "duration" :
                                                        duration_ms, "timeEnd" : end_time_ms, "countToEnd" : entry
                                                        .count_to_end, "linkStart" : entry.link_start }
                                                    );
                                                    if let Some(ref gid) = group_id {
                                                        event_payload["parent"] = serde_json::json!(gid);
                                                    }
                                                    if let Some(ref a) = internal_after_id {
                                                        event_payload["after"] = serde_json::json!(a);
                                                    }
                                                    logs.write()
                                                        .push(
                                                            format!(
                                                                "[{}] Entry: {}",
                                                                chrono::Local::now().format("%H:%M:%S"),
                                                                entry.name,
                                                            ),
                                                        );
                                                    match client
                                                        .post(&endpoint)
                                                        .header("Content-Type", "application/json")
                                                        .json(&event_payload)
                                                        .send()
                                                        .await
                                                    {
                                                        Ok(resp) => {
                                                            let status = resp.status();
                                                            if status.is_success() {
                                                                if let Ok(json) = resp.json::<serde_json::Value>().await {
                                                                    if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                                                                        internal_after_id = Some(id.to_string());
                                                                        logs.write()
                                                                            .push(
                                                                                format!(
                                                                                    "[{}] ✓ Created: {}",
                                                                                    chrono::Local::now().format("%H:%M:%S"),
                                                                                    id,
                                                                                ),
                                                                            );
                                                                    }
                                                                }
                                                            } else {
                                                                let body = resp.text().await.unwrap_or_default();
                                                                logs.write()
                                                                    .push(
                                                                        format!(
                                                                            "[{}] ✗ Error {}: {}",
                                                                            chrono::Local::now().format("%H:%M:%S"),
                                                                            status,
                                                                            body,
                                                                        ),
                                                                    );
                                                            }
                                                        }
                                                        Err(e) => {
                                                            logs.write()
                                                                .push(
                                                                    format!(
                                                                        "[{}] ✗ Request failed: {}",
                                                                        chrono::Local::now().format("%H:%M:%S"),
                                                                        e,
                                                                    ),
                                                                );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                    logs.write()
                                        .push(
                                            format!(
                                                "[{}] Push Complete - refreshing timeline...",
                                                chrono::Local::now().format("%H:%M:%S"),
                                            ),
                                        );
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
                                                let entry_time_end = entry.time_end;

        

                                                let is_referenced = formatter_items
                                                    .read()
                                                    .iter()
                                                    .any(|item| {
                                                        matches!(
                                                            item,
                                                            FormatterItem::Reference { id, .. }
                                                            if *id == entry_id_clone
                                                        )
                                                    });
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
                                                            let is_already_referenced = formatter_items
                
        
                                                                .read()
                                                                .iter()
                                                                .any(|item| {
                                                                    matches!(
                                                                        item,
                                                                        FormatterItem::Reference { id, .. }
                                                                        if *id == entry_id_clone
                                                                    )
                                                                });
                                                            if is_already_referenced {
                                                                return;
                                                            }
                                                            let mode = if entry_type == "group" {
                                                                InsertionMode::Into
                                                            } else {
                                                                InsertionMode::After
                                                            };
                                                            logs.write()
                                                                .push(
                                                                    format!(
                                                                        "[{}] Added Reference: {}",
                                                                        chrono::Local::now().format("%H:%M:%S"),
                                                                        entry_title_clone,
                                                                    ),
                                                                );
                                                            formatter_items
                                                                .write()
                                                                .push(FormatterItem::Reference {
                                                                    id: entry_id_clone.clone(),
                                                                    title: entry_title_clone.clone(),
                                                                    item_type: entry_type.clone(),
                                                                    mode,
                                                                    time_end: entry_time_end,
                                                                });
                                                        },
                                                        div { class: "entry-main",
                                                            if !entry.cue.is_empty() {
                                                                span { class: "entry-cue", "{entry.cue}" }
                                                            }
                                                            span { class: "entry-title", "{entry.title}" }
                                                            if entry.duration > 0 {
                                                                span { class: "entry-duration", "{format_ms_to_duration(entry.duration)}" }
                                                            }
                                                            if entry.time_end > 0 {
                                                                span { class: "entry-end-time", "→ {format_ms_to_duration(entry.time_end)}" }
                                                            }
                                                            if is_referenced {
                                                                div { style: "margin-left: auto; color: var(--accent-ot); font-weight: bold;",
                                                                    "✓ REF"
                                                                }
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
            // Time Picker Modal Render
            if let Some(ctx) = active_time_edit.read().clone() {
                TimePicker {
                    value: ctx.current_value,
                    field: ctx.field,
                    on_close: move |_| active_time_edit.set(None),
                    on_save: move |new_val: String| {
                        let mut items = formatter_items.write();
                        if let Some(item) = items.get_mut(ctx.item_idx) {
                            match item {
                                FormatterItem::Standalone(entry) => {
                                    match ctx.field {
                                        TimeField::Duration => entry.duration = new_val,
                                        TimeField::EndTime => entry.end_time = new_val,
                                    }
                                }
                                FormatterItem::Group { entries, .. } => {
                                    if let Some(sub_idx) = ctx.sub_item_idx {
                                        if let Some(entry) = entries.get_mut(sub_idx) {
                                            match ctx.field {
                                                TimeField::Duration => entry.duration = new_val,
                                                TimeField::EndTime => entry.end_time = new_val,
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        active_time_edit.set(None);
                    },
                }
            }
        }
    }
}
