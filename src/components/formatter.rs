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
    let mut show_logs = use_signal(|| true);
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
        div { class: "h-full flex flex-col p-6 gap-6 overflow-hidden bg-zinc-950 font-mono",
            // Connection Status Header
            div { class: "flex gap-4 shrink-0",
                div { class: "flex items-center gap-3 text-[0.7rem] font-bold text-zinc-400 uppercase bg-zinc-900/50 py-2 px-3 rounded border border-zinc-800 shadow-sm",
                    if let Some(Ok(_)) = &*playlists_resource.read() {
                        div { class: "w-2 h-2 rounded-full shadow-[0_0_8px_currentColor] animate-pulse text-emerald-500", }
                        "ProPresenter: Connected"
                    } else {
                        div { class: "w-2 h-2 rounded-full shadow-[0_0_8px_currentColor] animate-pulse text-red-500", }
                        "ProPresenter: Disconnected"
                    }
                }
                div { class: "flex items-center gap-3 text-[0.7rem] font-bold text-zinc-400 uppercase bg-zinc-900/50 py-2 px-3 rounded border border-zinc-800 shadow-sm",
                    if let Some(Ok(_)) = &*ontime_resource.read() {
                        div { class: "w-2 h-2 rounded-full shadow-[0_0_8px_currentColor] animate-pulse text-emerald-500", }
                        "Ontime: Connected"
                    } else {
                        div { class: "w-2 h-2 rounded-full shadow-[0_0_8px_currentColor] animate-pulse text-red-500", }
                        "Ontime: Disconnected"
                    }
                }
            }

            div { class: "flex-1 min-h-0 w-full grid grid-cols-[320px_1fr_340px] gap-6",
                // Left Panel - Playlist Source
                div { class: "bg-zinc-900/80 border border-zinc-800/80 rounded-lg flex flex-col overflow-hidden shadow-lg backdrop-blur-sm",
                    div { class: "p-4 text-xs font-extrabold tracking-widest text-zinc-500 border-b border-zinc-800/80 bg-zinc-950/30 uppercase flex justify-between items-center", "PLAYLIST SOURCE" }
                    div { class: "p-4 border-b border-zinc-800/50",
                        label { class: "text-[0.7rem] font-bold text-zinc-500 tracking-wider mb-2 block uppercase", "SELECT PLAYLIST" }
                        div { class: "flex gap-2",
                            select {
                                class: "flex-1 bg-zinc-950 border border-zinc-700/50 text-zinc-200 p-2.5 rounded font-mono text-sm focus:outline-none focus:border-emerald-500/50 focus:bg-emerald-500/5 transition-all appearance-none cursor-pointer",
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
                                class: "w-10 h-10 flex items-center justify-center bg-zinc-800 border border-zinc-700 text-zinc-400 rounded hover:text-emerald-500 hover:border-emerald-500/50 hover:bg-emerald-500/10 transition-all cursor-pointer text-lg",
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
                    div { class: "flex-1 overflow-y-auto p-2 space-y-1 scrollbar-thin scrollbar-thumb-zinc-700 scrollbar-track-transparent",
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
                                                class: if is_added { "flex items-center gap-3 p-2.5 rounded-md cursor-pointer transition-all border opacity-50 bg-emerald-500/5 border-emerald-500/20 hover:opacity-70" } else { "flex items-center gap-3 p-2.5 rounded-md cursor-pointer transition-all border border-transparent hover:bg-zinc-800 hover:border-zinc-700" },
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
                                                    span { class: "w-5 h-5 flex items-center justify-center bg-emerald-500 text-black rounded-full text-[0.65rem] font-extrabold shrink-0", "✓" }
                                                }
                                                span { class: "text-zinc-600 text-[0.7rem] font-mono min-w-[20px] text-right", "{item.id.index + 1}" }
                                                span { class: "flex-1 truncate text-sm text-zinc-200", "{item.id.name}" }
                                                span { class: "text-[0.6rem] text-zinc-500 uppercase tracking-wider", "{item.item_type}" }
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
                div { class: "bg-zinc-900/80 border border-zinc-800/80 rounded-lg flex flex-col overflow-hidden shadow-lg backdrop-blur-sm",
                    div { class: "p-4 text-xs font-extrabold tracking-widest text-zinc-500 border-b border-zinc-800/80 bg-zinc-950/30 uppercase flex justify-between items-center", "ONTIME FORMATTER" }
                    // Insertion selector removed - replaced by Reference Items list logic
                    div { class: "insertion-selector",
                        div { style: "color: var(--text-muted); font-size: 0.8rem; padding: 0 0 10px 0;",
                            "Click items in Timeline to add insertion points."
                        }
                    }
                    // Timeline Panel - removed placeholder
                    button {
                        class: "m-4 p-3 border-2 border-dashed border-zinc-700 rounded-lg text-zinc-500 font-mono text-xs font-bold hover:border-emerald-500/50 hover:text-emerald-500 hover:bg-emerald-500/5 transition-all w-[calc(100%-2rem)] cursor-pointer",
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
                    div { class: "flex-1 overflow-y-auto px-4 pb-4 space-y-3 scrollbar-thin scrollbar-thumb-zinc-700 scrollbar-track-transparent",
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
                                                format!("flex flex-col items-stretch p-3 rounded-md bg-zinc-950/50 border border-zinc-800/50 mb-1 backdrop-blur-sm cursor-grab active:cursor-grabbing dragging{}", drag_over_class)
                                            } else { 
                                                format!("flex flex-col items-stretch p-3 rounded-md bg-zinc-950/50 border border-zinc-800/50 mb-1 backdrop-blur-sm cursor-grab active:cursor-grabbing{}", drag_over_class)
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
                                            div { class: "flex items-center mb-2.5",
                                                span { class: "text-zinc-600 cursor-grab mr-2 select-none font-bold", "⋮⋮" }
                                                span { class: "flex-1 text-sm text-zinc-200 truncate", "{entry.name}" }
                                                button {
                                                    class: "w-7 h-7 flex items-center justify-center rounded text-zinc-500 hover:bg-red-500/10 hover:border-red-500/30 hover:text-red-500 transition-all font-mono text-lg",
                                                    onclick: move |_| {
                                                        formatter_items.write().remove(item_idx);
                                                    },
                                                    "×"
                                                }
                                            }
                                            div { class: "flex gap-3 flex-wrap pl-1",
                                                div { class: "flex flex-col gap-1",
                                                    label { class: "text-[0.6rem] uppercase text-zinc-500 font-bold", "Duration" }
                                                    input {
                                                        r#type: "text",
                                                        class: "w-20 p-1.5 rounded bg-zinc-900 border border-zinc-800 text-zinc-200 font-mono text-xs text-center focus:border-cyan-500 focus:outline-none cursor-pointer",
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
                                                div { class: "flex flex-col gap-1",
                                                    label { class: "text-[0.6rem] uppercase text-zinc-500 font-bold", "End Time" }
                                                    input {
                                                        r#type: "text",
                                                        class: "w-20 p-1.5 rounded bg-zinc-900 border border-zinc-800 text-zinc-200 font-mono text-xs text-center focus:border-cyan-500 focus:outline-none cursor-pointer",
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
                                                div { class: "flex items-center gap-1.5",
                                                    input {
                                                        r#type: "checkbox",
                                                        id: "cte-{item_idx}",
                                                        class: "w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-cyan-500 focus:ring-0 focus:ring-offset-0 cursor-pointer",
                                                        checked: count_to_end,
                                                        onchange: move |e| {
                                                            if let FormatterItem::Standalone(ref mut ent) = &mut formatter_items
                                                                .write()[item_idx]
                                                            {
                                                                ent.count_to_end = e.checked();
                                                            }
                                                        },
                                                    }
                                                    label { r#for: "cte-{item_idx}", class: "text-xs text-zinc-400 font-bold cursor-pointer", "CTE" }
                                                }
                                                div { class: "flex items-center gap-1.5",
                                                    input {
                                                        r#type: "checkbox",
                                                        id: "ls-{item_idx}",
                                                        class: "w-4 h-4 rounded border-zinc-600 bg-zinc-800 text-cyan-500 focus:ring-0 focus:ring-offset-0 cursor-pointer",
                                                        checked: link_start,
                                                        onchange: move |e| {
                                                            if let FormatterItem::Standalone(ref mut ent) = &mut formatter_items
                                                                .write()[item_idx]
                                                            {
                                                                ent.link_start = e.checked();
                                                            }
                                                        },
                                                    }
                                                    label { r#for: "ls-{item_idx}", class: "text-xs text-zinc-400 font-bold cursor-pointer", "Link" }
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
                                                format!("bg-zinc-950 border border-zinc-800 rounded-lg mb-3 cursor-pointer transition-all overflow-hidden !border-cyan-500 shadow-[0_0_0_1px_#06b6d4,0_0_20px_rgba(6,182,212,0.1)] draggable{}", drag_over_class)
                                            } else { 
                                                format!("bg-zinc-950 border border-zinc-800 rounded-lg mb-3 cursor-pointer transition-all overflow-hidden hover:border-zinc-600 draggable{}", drag_over_class)
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
                                            div { class: "flex justify-between items-center p-4 border-b border-zinc-800 bg-zinc-900/50",
                                                span { class: "text-zinc-600 cursor-grab mr-2 select-none font-bold", "⋮⋮" }
                                                input {
                                                    class: "bg-transparent border border-transparent text-zinc-100 font-mono font-bold text-sm px-2 py-1 rounded flex-1 min-w-0 hover:bg-zinc-800 hover:border-zinc-700 focus:outline-none focus:bg-zinc-950 focus:border-cyan-500 focus:shadow-[0_0_0_2px_rgba(6,182,212,0.1)] transition-all",
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
                                                    class: "bg-black/30 border border-white/20 text-white font-mono text-center rounded px-2 py-1 w-20 text-xs",
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
                                                span { class: "text-xs text-zinc-500 ml-2", "{entry_count} items" }
                                                button {
                                                    class: "w-7 h-7 flex items-center justify-center rounded text-zinc-500 hover:bg-red-500/10 hover:border-red-500/30 hover:text-red-500 transition-all font-mono text-lg ml-2",
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
                                                " border-b-2 border-b-cyan-500"
                                            } else {
                                                " border-t-2 border-t-cyan-500"
                                            }
                                        } else {
                                            ""
                                        }
                                    } else {
                                        ""
                                    };

                                    rsx! {
                                        div { 
                                            class: format!("pl-4 relative before:content-[''] before:absolute before:left-0 before:top-0 before:bottom-0 before:w-1 before:bg-zinc-800 flex flex-col items-stretch p-3 rounded-md bg-zinc-950/50 border border-zinc-800/50 mb-1 backdrop-blur-sm cursor-grab active:cursor-grabbing{}", drag_over_class),
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
                                            div { class: "flex items-center text-sm",
                                                div { class: "text-cyan-500 font-bold mr-2 w-4 text-center",
                                                    if matches!(mode, InsertionMode::Into) {
                                                        "↳"
                                                    } else {
                                                        "↓"
                                                    }
                                                }
                                                div { class: "flex-1 flex items-center gap-2 overflow-hidden",
                                                    span { class: "text-zinc-600 cursor-grab select-none font-bold", "⋮⋮" }
                                                    span { class: "text-[0.6rem] font-bold px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400",
                                                        if matches!(mode, InsertionMode::Into) {
                                                            "INSERT INTO"
                                                        } else {
                                                            "INSERT AFTER"
                                                        }
                                                    }
                                                    span { class: "flex-1 truncate text-zinc-300 italic", "{title}" }
                                                }
                                                button {
                                                    class: "w-7 h-7 flex items-center justify-center rounded text-zinc-500 hover:bg-red-500/10 hover:border-red-500/30 hover:text-red-500 transition-all font-mono text-lg ml-2",
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
                    }
                    // Push to OnTime button
                    div { class: "p-4 border-t border-zinc-800/50",
                        button {
                            class: "w-full py-4 bg-cyan-600 hover:bg-cyan-500 text-white rounded font-bold tracking-wider uppercase transition-all shadow-[0_0_20px_rgba(8,145,178,0.3)] hover:shadow-[0_0_30px_rgba(8,145,178,0.5)] transform hover:-translate-y-0.5",
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
                div { class: "bg-zinc-900/80 border border-zinc-800/80 rounded-lg flex flex-col overflow-hidden shadow-lg backdrop-blur-sm hover:border-cyan-500/50 transition-colors group",
                    div { class: "p-4 text-xs font-extrabold tracking-widest text-zinc-500 border-b border-zinc-800/80 bg-zinc-950/30 uppercase flex justify-between items-center group-hover:text-cyan-500 transition-colors",
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
                                    div { class: "flex items-center border-b border-zinc-800/50 py-1.5 last:border-0 hover:bg-zinc-800/30 transition-colors",
                                        span { class: "flex-1 font-bold truncate text-sm text-zinc-300", "{rundown.title}" }
                                        span { class: "text-[0.65rem] uppercase text-zinc-500 bg-zinc-900 px-1.5 py-0.5 rounded border border-zinc-800", "Rev: {rundown.revision}" }
                                    }
                                    div { class: "overflow-y-auto flex-1 pr-1 space-y-0.5 scrollbar-thin scrollbar-thumb-zinc-700 scrollbar-track-transparent",
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
                                                    "ml-4 flex flex-col p-2 rounded hover:bg-zinc-800/80 cursor-pointer border-b border-zinc-800/30 transition-all group"
                                                } else if entry.entry_type == "group" {
                                                    "flex flex-col p-2 rounded hover:bg-zinc-800/80 cursor-pointer border-b border-zinc-800/30 bg-zinc-900/30 mt-2 mb-1 group"
                                                } else {
                                                    "flex flex-col p-2 rounded hover:bg-zinc-800/80 cursor-pointer border-b border-zinc-800/30 transition-all group"
                                                };
                                                rsx! {
                                                    div {
                                                        class: if is_referenced { format!("{} !border-r-4 !border-r-emerald-500 bg-emerald-500/5", base_class) } else { base_class.to_string() },
                                                        style: if !entry.colour.is_empty() { format!("border-left: 3px solid {};", entry.colour) } else { "border-left: 3px solid transparent;".to_string() },
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
                                                        div { class: "flex items-center gap-2",
                                                            if !entry.cue.is_empty() {
                                                                span { class: "text-[0.65rem] font-bold bg-zinc-800 text-zinc-400 px-1.5 rounded min-w-[24px] text-center", "{entry.cue}" }
                                                            }
                                                            span { class: "flex-1 truncate font-medium text-zinc-300 group-hover:text-white transition-colors", "{entry.title}" }
                                                            if entry.duration > 0 {
                                                                span { class: "text-[0.65rem] font-mono text-zinc-500", "{format_ms_to_duration(entry.duration)}" }
                                                            }
                                                            if entry.time_end > 0 {
                                                                span { class: "text-[0.65rem] font-mono text-zinc-600", "→ {format_ms_to_duration(entry.time_end)}" }
                                                            }
                                                            if is_referenced {
                                                                div { class: "ml-auto text-emerald-500 font-bold text-[0.6rem] uppercase tracking-wider bg-emerald-500/10 px-1.5 rounded border border-emerald-500/20",
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
            div { 
                class: if show_logs() { 
                    "bg-zinc-950 border border-zinc-800 rounded-lg flex flex-col overflow-hidden shrink-0 h-[200px] min-h-[150px] max-h-[30vh] transition-all" 
                } else { 
                    "bg-zinc-950 border border-zinc-800 rounded-lg flex flex-col overflow-hidden shrink-0 h-[38px] min-h-0 flex-none border-b-0 transition-all" 
                },
                div { class: "p-2.5 px-4 bg-zinc-900 border-b border-zinc-800 text-xs font-bold text-zinc-400 uppercase tracking-wider flex justify-between items-center select-none cursor-pointer hover:text-zinc-200 hover:bg-zinc-800",
                    onclick: move |_| show_logs.set(!show_logs()),
                    "LIVE LOGS"
                    button {
                        class: "w-5 h-5 flex items-center justify-center rounded hover:bg-zinc-700 text-zinc-400 transition-colors",
                        title: if show_logs() { "Hide Logs" } else { "Show Logs" },
                        if show_logs() { "▼" } else { "▲" }
                    }
                }
                if show_logs() {
                    div { class: "flex-1 overflow-y-auto bg-zinc-950 p-2 font-mono text-xs space-y-0.5 scrollbar-thin scrollbar-thumb-zinc-700 scrollbar-track-transparent",
                        for log in logs.read().iter() {
                            div { class: "text-xs font-mono py-0.5 text-zinc-400 border-b border-zinc-800/30 last:border-0 hover:bg-zinc-900/50 hover:text-zinc-200 break-all", "{log}" }
                        }
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
