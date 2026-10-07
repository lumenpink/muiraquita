use dioxus::prelude::*;

#[component]
pub fn ContentWarningOverlay(warning: String, children: Element) -> Element {
    let mut is_revealed = use_signal(|| false);

    rsx! {
        div { class: "cw-container",
            div { class: "cw-banner",
                span { class: "cw-icon", "⚠️" }
                span { class: "cw-text", "Aviso de Conteúdo: {warning}" }
                button {
                    class: "cw-toggle-btn",
                    onclick: move |_| is_revealed.toggle(),
                    if *is_revealed.read() {
                        "Ocultar"
                    } else {
                        "Mostrar"
                    }
                }
            }
            div {
                class: if *is_revealed.read() { "cw-content revealed" } else { "cw-content blurred" },
                {children}
            }
        }
    }
}
