use dioxus::prelude::*;
use crate::Route;

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav {
            class: "h-16 bg-zinc-950 border-b border-zinc-800 flex items-center justify-between px-8 shrink-0",
            div { class: "flex items-center gap-3 font-mono",
                span { class: "text-emerald-500 font-extrabold text-xl tracking-wider drop-shadow-[0_0_8px_rgba(16,185,129,0.4)]", "PP2OT" }
                span { class: "text-zinc-100 font-bold text-lg tracking-wide", "BRIDGE" }
            }
            div { class: "flex gap-2 p-1 bg-zinc-900/50 rounded-lg border border-zinc-800/50",
                Link {
                    to: Route::Formatter {},
                    class: "px-4 py-2 rounded text-xs font-bold text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800 transition-all uppercase tracking-widest",
                    active_class: "!bg-zinc-800 !text-emerald-400 shadow-sm",
                    "FORMATTER"
                }
                Link {
                    to: Route::Settings {},
                    class: "px-4 py-2 rounded text-xs font-bold text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800 transition-all uppercase tracking-widest",
                    active_class: "!bg-zinc-800 !text-emerald-400 shadow-sm",
                    "SETTINGS"
                }
            }
        }
        div {
            class: "flex-1 overflow-hidden relative bg-zinc-950 flex flex-col font-mono",
            Outlet::<Route> {} 
        }
    }
}
