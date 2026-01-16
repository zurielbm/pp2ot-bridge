use dioxus::prelude::*;
use crate::Route;

/// Shared navbar component.
#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav { id: "navbar",
            div { class: "nav-brand",
                span { class: "accent", "PP2OT" }
                " BRIDGE"
            }
            div { class: "nav-links",
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
