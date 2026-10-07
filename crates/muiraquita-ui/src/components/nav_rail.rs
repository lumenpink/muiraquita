use crate::state::ActiveTab;
use dioxus::prelude::*;

#[component]
pub fn NavRail(
    active: ActiveTab,
    on_change: EventHandler<ActiveTab>,
) -> Element {
    rsx! {
        nav { class: "nav-rail",
            button {
                class: if active == ActiveTab::Timeline { "nav-btn active" } else { "nav-btn" },
                onclick: move |_| on_change.call(ActiveTab::Timeline),
                span { class: "nav-icon", "📰" }
                span { class: "nav-label", "Timeline" }
            }
            button {
                class: if active == ActiveTab::Channels { "nav-btn active" } else { "nav-btn" },
                onclick: move |_| on_change.call(ActiveTab::Channels),
                span { class: "nav-icon", "📺" }
                span { class: "nav-label", "Canais" }
            }
            button {
                class: if active == ActiveTab::Compose { "nav-btn active" } else { "nav-btn" },
                onclick: move |_| on_change.call(ActiveTab::Compose),
                span { class: "nav-icon", "✍️" }
                span { class: "nav-label", "Compor" }
            }
            button {
                class: if active == ActiveTab::Settings { "nav-btn active" } else { "nav-btn" },
                onclick: move |_| on_change.call(ActiveTab::Settings),
                span { class: "nav-icon", "⚙️" }
                span { class: "nav-label", "Ajustes" }
            }
        }
    }
}
