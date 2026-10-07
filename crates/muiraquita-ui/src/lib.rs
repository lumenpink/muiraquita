pub mod components;
pub mod state;

use components::*;
use dioxus::prelude::*;
use muiraquita_adapters::fakes::FakeSocialStore;
use muiraquita_domain::micropub::MicropubDraft;
use muiraquita_domain::microsub::InteractionType;
use muiraquita_domain::ports::{AuthPort, MicropubPort, MicrosubPort};
use state::{ActiveTab, AppState};
use url::Url;

/// Estilos CSS embutidos alinhados com a identidade visual do ecossistema Baquara e IndieInABox
pub const APP_STYLES: &str = r#"
:root {
  --bg-color: #0d1117;
  --card-bg: #161b22;
  --card-border: #30363d;
  --text-main: #c9d1d9;
  --text-bright: #f0f6fc;
  --text-muted: #8b949e;
  --primary: #2ea043;
  --primary-hover: #3fb950;
  --accent: #58a6ff;
  --cw-bg: #382400;
  --cw-border: #bb8009;
  --cw-text: #f0883e;
  --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  --font-mono: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
}

body {
  margin: 0;
  background-color: var(--bg-color);
  color: var(--text-main);
  font-family: var(--font-sans);
  line-height: 1.5;
}

.muiraquita-root {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

/* Header */
.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1.5rem;
  background-color: var(--card-bg);
  border-bottom: 1px solid var(--card-border);
}
.brand-container {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.brand-logo {
  font-size: 2rem;
}
.brand-title {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--text-bright);
}
.brand-subtitle {
  font-size: 0.8rem;
  color: var(--text-muted);
}
.user-profile-badge {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}
.status-indicator.online {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--primary);
  display: inline-block;
}
.user-me-link {
  color: var(--accent);
  text-decoration: none;
  font-size: 0.9rem;
  font-family: var(--font-mono);
}

/* Layout */
.main-layout {
  display: flex;
  flex: 1;
}

/* Navigation Rail */
.nav-rail {
  width: 200px;
  background-color: #010409;
  border-right: 1px solid var(--card-border);
  padding: 1rem 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}
.nav-btn {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.6rem 0.8rem;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--text-main);
  cursor: pointer;
  font-size: 0.95rem;
  text-align: left;
  transition: all 0.15s ease;
}
.nav-btn:hover {
  background-color: var(--card-bg);
  color: var(--text-bright);
}
.nav-btn.active {
  background-color: var(--card-bg);
  border-color: var(--primary);
  color: var(--primary);
  font-weight: 600;
}

/* Content Area */
.content-area {
  flex: 1;
  padding: 1.5rem;
  max-width: 720px;
  margin: 0 auto;
  width: 100%;
  box-sizing: border-box;
}

/* Entries & Cards */
.entry-card {
  background-color: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 8px;
  padding: 1.25rem;
  margin-bottom: 1.25rem;
  box-shadow: 0 2px 6px rgba(0,0,0,0.2);
}
.entry-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.85rem;
}
.entry-author-box {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}
.author-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  object-fit: cover;
}
.author-avatar-fallback {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background-color: #21262d;
  color: var(--primary);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
}
.author-meta {
  display: flex;
  flex-direction: column;
}
.author-name {
  color: var(--text-bright);
  font-weight: 600;
  font-size: 0.95rem;
  text-decoration: none;
}
.entry-date {
  font-size: 0.75rem;
  color: var(--text-muted);
}
.network-badge {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.2rem 0.5rem;
  background-color: #21262d;
  border: 1px solid var(--card-border);
  border-radius: 12px;
  font-size: 0.75rem;
  color: var(--text-muted);
}
.network-origin {
  color: var(--accent);
  font-family: var(--font-mono);
}
.entry-text {
  margin: 0.5rem 0;
  white-space: pre-wrap;
}
.entry-tags {
  display: flex;
  gap: 0.4rem;
  margin-top: 0.6rem;
}
.tag-pill {
  font-size: 0.75rem;
  color: var(--accent);
  background-color: rgba(88,166,255,0.1);
  padding: 0.15rem 0.45rem;
  border-radius: 4px;
}

/* Action Toolbar */
.entry-footer {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--card-border);
}
.action-btn {
  background: transparent;
  border: 1px solid var(--card-border);
  color: var(--text-main);
  padding: 0.35rem 0.65rem;
  border-radius: 6px;
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.15s ease;
}
.action-btn:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--accent);
}
.action-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* Content Warning */
.cw-container {
  margin: 0.5rem 0;
}
.cw-banner {
  background-color: var(--cw-bg);
  border: 1px solid var(--cw-border);
  color: var(--cw-text);
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.85rem;
  font-weight: 600;
}
.cw-toggle-btn {
  background: #221500;
  border: 1px solid var(--cw-border);
  color: var(--cw-text);
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  cursor: pointer;
}
.cw-content.blurred {
  filter: blur(8px);
  pointer-events: none;
  user-select: none;
  max-height: 80px;
  overflow: hidden;
}
.cw-content.revealed {
  filter: none;
  margin-top: 0.5rem;
}

/* Polls */
.poll-container {
  background-color: #010409;
  border: 1px solid var(--card-border);
  border-radius: 6px;
  padding: 0.75rem;
  margin: 0.75rem 0;
}
.poll-header {
  font-weight: 600;
  font-size: 0.85rem;
  margin-bottom: 0.5rem;
  color: var(--text-bright);
}
.poll-option-row {
  margin-bottom: 0.5rem;
}
.poll-option-info {
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
  margin-bottom: 0.2rem;
}
.poll-bar-track {
  width: 100%;
  height: 8px;
  background-color: #21262d;
  border-radius: 4px;
  overflow: hidden;
}
.poll-bar-fill {
  height: 100%;
  background-color: var(--primary);
  border-radius: 4px;
  transition: width 0.3s ease;
}
.poll-vote-btn {
  margin-top: 0.3rem;
  background: var(--card-bg);
  border: 1px solid var(--primary);
  color: var(--primary);
  padding: 0.2rem 0.6rem;
  border-radius: 4px;
  font-size: 0.75rem;
  cursor: pointer;
}

/* Buttons & Inputs */
.btn-primary {
  background-color: var(--primary);
  color: #fff;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
}
.btn-primary:hover:not(:disabled) {
  background-color: var(--primary-hover);
}
.btn-secondary {
  background-color: var(--card-bg);
  border: 1px solid var(--card-border);
  color: var(--text-main);
  padding: 0.5rem 1rem;
  border-radius: 6px;
  cursor: pointer;
}
.btn-sm {
  padding: 0.3rem 0.6rem;
  font-size: 0.8rem;
}
.input-field, .composer-textarea {
  width: 100%;
  background-color: #010409;
  border: 1px solid var(--card-border);
  color: var(--text-bright);
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  box-sizing: border-box;
  font-family: inherit;
  font-size: 0.9rem;
}
.input-field:focus, .composer-textarea:focus {
  border-color: var(--accent);
  outline: none;
}

/* Composer */
.composer-box {
  background-color: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 8px;
  padding: 1.25rem;
}
.composer-title {
  margin-top: 0;
  font-size: 1.15rem;
  color: var(--text-bright);
}
.composer-tabs {
  display: flex;
  gap: 0.4rem;
  margin-bottom: 0.8rem;
}
.composer-tabs .tab-btn {
  background: transparent;
  border: 1px solid var(--card-border);
  color: var(--text-muted);
  padding: 0.3rem 0.7rem;
  border-radius: 6px;
  cursor: pointer;
}
.composer-tabs .tab-btn.active {
  border-color: var(--primary);
  color: var(--primary);
  font-weight: 600;
}
.composer-field {
  margin-bottom: 0.75rem;
}
.composer-options {
  display: flex;
  gap: 1rem;
  margin-bottom: 0.75rem;
}
.checkbox-label {
  font-size: 0.85rem;
  display: flex;
  align-items: center;
  gap: 0.4rem;
  cursor: pointer;
}
.composer-poll-builder {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
  padding: 0.6rem;
  background: #010409;
  border-radius: 6px;
}

/* Modal */
.modal-backdrop {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background-color: rgba(0,0,0,0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
.modal-dialog {
  background-color: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 8px;
  width: 90%;
  max-width: 480px;
  padding: 1.5rem;
  box-shadow: 0 8px 24px rgba(0,0,0,0.5);
}
.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}
.modal-header h3 { margin: 0; color: var(--text-bright); }
.modal-close-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 1.2rem;
  cursor: pointer;
}
.modal-help-text {
  font-size: 0.8rem;
  color: var(--text-muted);
  margin-top: 0.5rem;
}
.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.6rem;
  margin-top: 1.25rem;
}

/* Footer GOST */
.site-footer {
  padding: 1.25rem;
  text-align: center;
  font-size: 0.8rem;
  color: var(--text-muted);
  border-top: 1px solid var(--card-border);
  background-color: var(--card-bg);
}
.footer-line a {
  color: var(--accent);
  text-decoration: none;
}
"#;

#[component]
pub fn App() -> Element {
    let mut state = use_signal(AppState::new);
    let store = use_signal(FakeSocialStore::seeded);

    // Initial load of channels and timeline
    use_effect(move || {
        let s = store.read().clone();
        let dummy = Url::parse("https://lumen.pink/microsub").unwrap();
        // Load channels
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(chans) = s.get_channels(&dummy, "token").await {
                state.write().channels.set(chans.channels);
            }
            let req = muiraquita_domain::microsub::MicrosubRequest::Timeline {
                channel: "timeline".to_string(),
                after: None,
                before: None,
                limit: None,
            };
            if let Ok(tl) = s.get_timeline(&dummy, "token", &req).await {
                state.write().entries.set(tl.items);
            }
        });
    });

    let current_tab = *state.read().active_tab.read();
    let current_session = state.read().session.read().clone();
    let channels = state.read().channels.read().clone();
    let entries = state.read().entries.read().clone();
    let modal_open = *state.read().auth_modal_open.read();

    // Event handlers
    let handle_like = move |url: String| {
        let s = store.read().clone();
        let dummy = Url::parse("https://lumen.pink/microsub").unwrap();
        wasm_bindgen_futures::spawn_local(async move {
            let req = muiraquita_domain::microsub::MicrosubRequest::Interact {
                interaction_type: InteractionType::Like,
                url,
                content: None,
                poll_option: None,
            };
            let _ = s.send_action(&dummy, "token", &req).await;
        });
    };

    let handle_repost = move |url: String| {
        let s = store.read().clone();
        let dummy = Url::parse("https://lumen.pink/microsub").unwrap();
        wasm_bindgen_futures::spawn_local(async move {
            let req = muiraquita_domain::microsub::MicrosubRequest::Interact {
                interaction_type: InteractionType::Repost,
                url,
                content: None,
                poll_option: None,
            };
            let _ = s.send_action(&dummy, "token", &req).await;
        });
    };

    let handle_vote = move |(url, opt_idx): (String, usize)| {
        let s = store.read().clone();
        let dummy = Url::parse("https://lumen.pink/microsub").unwrap();
        wasm_bindgen_futures::spawn_local(async move {
            let req = muiraquita_domain::microsub::MicrosubRequest::Interact {
                interaction_type: InteractionType::PollVote,
                url,
                content: None,
                poll_option: Some(opt_idx),
            };
            let _ = s.send_action(&dummy, "token", &req).await;
            // Refresh timeline
            let tl_req = muiraquita_domain::microsub::MicrosubRequest::Timeline {
                channel: "timeline".to_string(),
                after: None,
                before: None,
                limit: None,
            };
            if let Ok(tl) = s.get_timeline(&dummy, "token", &tl_req).await {
                state.write().entries.set(tl.items);
            }
        });
    };

    let handle_publish = move |draft: MicropubDraft| {
        let s = store.read().clone();
        let dummy = Url::parse("https://lumen.pink/micropub").unwrap();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = s.create_post(&dummy, "token", &draft).await;
            // Refresh timeline
            let tl_req = muiraquita_domain::microsub::MicrosubRequest::Timeline {
                channel: "timeline".to_string(),
                after: None,
                before: None,
                limit: None,
            };
            if let Ok(tl) = s.get_timeline(&dummy, "token", &tl_req).await {
                state.write().entries.set(tl.items);
                state.write().active_tab.set(ActiveTab::Timeline);
            }
        });
    };

    let handle_login = move |domain: String| {
        let s = store.read().clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(parsed) = Url::parse(&domain) {
                let pkce = muiraquita_domain::indieauth::PkcePair::mock_for_testing();
                let dummy_token_ep = Url::parse("https://tokens.indieauth.com/token").unwrap();
                let client_id = Url::parse("https://muiraquita.2lp.in/").unwrap();
                let redirect_uri = Url::parse("https://muiraquita.2lp.in/callback").unwrap();

                if let Ok(sess) = s
                    .exchange_code(&dummy_token_ep, "fake-code", &parsed, &redirect_uri, &client_id, &pkce)
                    .await
                {
                    state.write().session.set(Some(sess));
                    state.write().auth_modal_open.set(false);
                }
            }
        });
    };

    rsx! {
        style { "{APP_STYLES}" }
        div { class: "muiraquita-root",
            Header {
                session: current_session,
                on_open_auth: move |_| state.write().auth_modal_open.set(true),
                on_logout: move |_| state.write().session.set(None),
            }

            div { class: "main-layout",
                NavRail {
                    active: current_tab,
                    on_change: move |tab| state.write().active_tab.set(tab),
                }

                main { class: "content-area",
                    match current_tab {
                        ActiveTab::Timeline => rsx! {
                            div { class: "timeline-view",
                                if entries.is_empty() {
                                    div { class: "entry-card",
                                        p { "Nenhum post na timeline no momento." }
                                    }
                                } else {
                                    for entry in entries {
                                        EntryCard {
                                            key: "{entry.uid.as_deref().unwrap_or(\"\")}",
                                            entry: entry,
                                            on_like: handle_like,
                                            on_repost: handle_repost,
                                            on_vote: handle_vote,
                                        }
                                    }
                                }
                            }
                        },
                        ActiveTab::Channels => rsx! {
                            div { class: "channels-view",
                                h2 { "📺 Canais Microsub" }
                                for ch in channels {
                                    div { class: "entry-card",
                                        h3 { "{ch.name}" }
                                        p { class: "entry-date", "UID: {ch.uid}" }
                                    }
                                }
                            }
                        },
                        ActiveTab::Compose => rsx! {
                            Composer {
                                on_publish: handle_publish,
                            }
                        },
                        ActiveTab::Settings => rsx! {
                            div { class: "settings-view entry-card",
                                h2 { "⚙️ Configurações & Conformidade GOST" }
                                p { strong { "Padrão: " } "GOST v1.1.0 (Perfil lumen)" }
                                p { strong { "XDG Config: " } "~/.config/muiraquita/config.json" }
                                p { strong { "XDG State: " } "~/.local/state/muiraquita/session.json" }
                                p { strong { "Framework UI: " } "Dioxus 0.8 (Signals nativos)" }
                                p { strong { "Client ID: " } "https://muiraquita.2lp.in/" }
                            }
                        },
                    }
                }
            }

            // IndieAuth Modal
            if modal_open {
                AuthModal {
                    on_login: handle_login,
                    on_close: move |_| state.write().auth_modal_open.set(false),
                }
            }

            // Rodapé canônico GOST 13
            footer { class: "site-footer",
                div { class: "footer-line",
                    "Orgulhosamente produzido por uma "
                    a { href: "https://travesti.2lp.in/pt/", target: "_blank", rel: "noopener", "travesti latina" }
                    " 🏳️‍⚧️ :)"
                }
                div { class: "footer-line",
                    "Muiraquitã v0.1.0 • por "
                    a { href: "https://lumen.pink", target: "_blank", rel: "noopener", "Lumen Pink" }
                    " • Brasil 🇧🇷 • "
                    a { href: "https://codeberg.org/lumenpink/muiraquita", target: "_blank", rel: "noopener", "GPL-3.0" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_mounts_complete_app() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Muiraquitã"));
        assert!(html.contains("IndieInABox Social Suite"));
        assert!(html.contains("Timeline"));
        assert!(html.contains("Compor"));
        assert!(html.contains("travesti latina"));
    }
}
