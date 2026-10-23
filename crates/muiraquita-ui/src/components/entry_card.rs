use crate::components::cw_overlay::ContentWarningOverlay;
use crate::components::poll_widget::PollWidget;
use dioxus::prelude::*;
use muiraquita_domain::jf2::Jf2Entry;

#[component]
pub fn EntryCard(
    entry: Jf2Entry,
    on_like: EventHandler<String>,
    on_repost: EventHandler<String>,
    on_vote: EventHandler<(String, usize)>,
) -> Element {
    let author_name = entry.author_name().to_string();
    let author_url = entry
        .author
        .as_ref()
        .and_then(|a| a.url.clone())
        .unwrap_or_default();
    let author_photo = entry.author.as_ref().and_then(|a| a.photo.clone());

    let entry_url = entry.url.clone().unwrap_or_default();
    let published = entry
        .published
        .clone()
        .unwrap_or_else(|| "Recente".to_string());
    let iiab = entry.iiab();
    let cw = iiab.content_warning.clone();
    let poll = iiab.poll.clone();
    let network = iiab.network.clone();
    let categories = entry.category.clone();

    let content_text = entry
        .content
        .as_ref()
        .map(|c| c.as_text().to_string())
        .unwrap_or_default();

    let entry_url_for_like = entry_url.clone();
    let entry_url_for_repost = entry_url.clone();
    let entry_url_for_vote = entry_url.clone();

    rsx! {
        article { class: "entry-card",
            // Header: Author & Network Badge
            header { class: "entry-header",
                div { class: "entry-author-box",
                    if let Some(photo) = author_photo {
                        img { class: "author-avatar", src: "{photo}", alt: "{author_name}" }
                    } else {
                        div { class: "author-avatar-fallback",
                            "{author_name.chars().next().unwrap_or('?')}"
                        }
                    }
                    div { class: "author-meta",
                        if !author_url.is_empty() {
                            a { class: "author-name", href: "{author_url}", target: "_blank", "{author_name}" }
                        } else {
                            span { class: "author-name", "{author_name}" }
                        }
                        span { class: "entry-date", "{published}" }
                    }
                }

                // IIAB Protocol & Origin Server Badge
                if let Some(net) = network {
                    div { class: "network-badge",
                        span { class: "network-icon", "{net.icon()}" }
                        span { class: "network-name", "{net.display_badge()}" }
                        if let Some(origin) = iiab.origin_server.as_ref() {
                            span { class: "network-origin", "@ {origin}" }
                        }
                    }
                }
            }

            // Body: Content Warning overlay or direct text
            div { class: "entry-body",
                if let Some(warning) = cw {
                    ContentWarningOverlay {
                        warning: warning,
                        p { class: "entry-text", "{content_text}" }
                    }
                } else {
                    p { class: "entry-text", "{content_text}" }
                }

                // Interactive Poll (IIAB extension)
                if let Some(p) = poll {
                    PollWidget {
                        poll: p,
                        can_vote: iiab.can_vote(),
                        on_vote: move |opt_idx| {
                            on_vote.call((entry_url_for_vote.clone(), opt_idx));
                        }
                    }
                }

                // Categories / Tags
                if !categories.is_empty() {
                    div { class: "entry-tags",
                        for tag in categories {
                            span { class: "tag-pill", "#{tag}" }
                        }
                    }
                }
            }

            // Footer: Interactive capabilities toolbar
            footer { class: "entry-footer",
                button {
                    class: "action-btn",
                    disabled: !iiab.can_like(),
                    title: if iiab.can_like() { "Curtir (Webmention/Microsub)" } else { "Não suportado na rede de origem" },
                    onclick: move |_| on_like.call(entry_url_for_like.clone()),
                    "❤️ Curtir"
                }
                button {
                    class: "action-btn",
                    disabled: !iiab.can_repost(),
                    title: if iiab.can_repost() { "Repostar" } else { "Não suportado na rede de origem" },
                    onclick: move |_| on_repost.call(entry_url_for_repost.clone()),
                    "🔄 Repostar"
                }
                button {
                    class: "action-btn",
                    disabled: !iiab.can_reply(),
                    title: if iiab.can_reply() { "Responder" } else { "Não suportado na rede de origem" },
                    "💬 Responder"
                }
            }
        }
    }
}
