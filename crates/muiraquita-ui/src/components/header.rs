use dioxus::prelude::*;
use muiraquita_domain::indieauth::AuthSession;

#[component]
pub fn Header(
    session: Option<AuthSession>,
    on_open_auth: EventHandler<()>,
    on_logout: EventHandler<()>,
) -> Element {
    rsx! {
        header { class: "app-header",
            div { class: "brand-container",
                span { class: "brand-logo", "🐸" }
                div { class: "brand-text",
                    h1 { class: "brand-title", "Muiraquitã" }
                    span { class: "brand-subtitle", "IndieWeb & IndieInABox Social Suite" }
                }
            }
            div { class: "header-actions",
                if let Some(s) = session {
                    div { class: "user-profile-badge",
                        span { class: "status-indicator online" }
                        a {
                            class: "user-me-link",
                            href: "{s.me.as_str()}",
                            target: "_blank",
                            "{s.me.as_str()}"
                        }
                        button {
                            class: "btn-secondary btn-sm",
                            onclick: move |_| on_logout.call(()),
                            "Sair"
                        }
                    }
                } else {
                    button {
                        class: "btn-primary btn-sm",
                        onclick: move |_| on_open_auth.call(()),
                        "🔑 Entrar com IndieAuth"
                    }
                }
            }
        }
    }
}
