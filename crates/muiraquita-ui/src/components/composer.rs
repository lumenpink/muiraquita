use dioxus::prelude::*;
use muiraquita_domain::iiab::{Poll, PollOption};
use muiraquita_domain::micropub::{MicropubDraft, PostKind};

#[component]
pub fn Composer(on_publish: EventHandler<MicropubDraft>) -> Element {
    let mut selected_kind = use_signal(|| PostKind::Note);
    let mut content = use_signal(String::new);
    let mut title = use_signal(String::new);
    let mut reply_to = use_signal(String::new);
    let mut cw_text = use_signal(String::new);
    let mut has_cw = use_signal(|| false);

    // Poll builder
    let mut has_poll = use_signal(|| false);
    let mut poll_opt_1 = use_signal(|| "Opção 1".to_string());
    let mut poll_opt_2 = use_signal(|| "Opção 2".to_string());

    let can_submit = !content.read().trim().is_empty();

    let submit_handler = move |_| {
        let text = content.read().clone();
        if text.trim().is_empty() {
            return;
        }

        let mut draft = match *selected_kind.read() {
            PostKind::Note => MicropubDraft::new_note(text),
            PostKind::Article => {
                let t = title.read().clone();
                MicropubDraft::new_article(t, text)
            }
            PostKind::Reply => {
                let target = reply_to.read().clone();
                MicropubDraft::new_reply(target, text)
            }
            _ => MicropubDraft::new_note(text),
        };

        if *has_cw.read() && !cw_text.read().trim().is_empty() {
            draft.content_warning = Some(cw_text.read().trim().to_string());
        }

        if *has_poll.read() {
            draft.poll = Some(Poll::new(
                vec![
                    PollOption { title: poll_opt_1.read().clone(), votes: 0 },
                    PollOption { title: poll_opt_2.read().clone(), votes: 0 },
                ],
                false,
            ));
        }

        on_publish.call(draft);
        content.set(String::new());
        title.set(String::new());
        reply_to.set(String::new());
        cw_text.set(String::new());
        has_cw.set(false);
        has_poll.set(false);
    };

    rsx! {
        div { class: "composer-box",
            h2 { class: "composer-title", "✍️ Compor Post (Micropub)" }

            // Kind selector tabs
            div { class: "composer-tabs",
                button {
                    class: if *selected_kind.read() == PostKind::Note { "tab-btn active" } else { "tab-btn" },
                    onclick: move |_| selected_kind.set(PostKind::Note),
                    "💬 Nota"
                }
                button {
                    class: if *selected_kind.read() == PostKind::Article { "tab-btn active" } else { "tab-btn" },
                    onclick: move |_| selected_kind.set(PostKind::Article),
                    "📄 Artigo"
                }
                button {
                    class: if *selected_kind.read() == PostKind::Reply { "tab-btn active" } else { "tab-btn" },
                    onclick: move |_| selected_kind.set(PostKind::Reply),
                    "↩️ Resposta"
                }
            }

            // Article title input
            if *selected_kind.read() == PostKind::Article {
                div { class: "composer-field",
                    input {
                        class: "input-field",
                        r#type: "text",
                        placeholder: "Título do Artigo...",
                        value: "{title}",
                        oninput: move |e| title.set(e.value()),
                    }
                }
            }

            // Reply target URL input
            if *selected_kind.read() == PostKind::Reply {
                div { class: "composer-field",
                    input {
                        class: "input-field",
                        r#type: "url",
                        placeholder: "URL do post original para responder (in-reply-to)...",
                        value: "{reply_to}",
                        oninput: move |e| reply_to.set(e.value()),
                    }
                }
            }

            // Content textarea
            div { class: "composer-field",
                textarea {
                    class: "composer-textarea",
                    placeholder: "O que você está pensando? (#hashtags são indexadas automaticamente)...",
                    rows: 4,
                    value: "{content}",
                    oninput: move |e| content.set(e.value()),
                }
            }

            // Options: CW & Poll
            div { class: "composer-options",
                label { class: "checkbox-label",
                    input {
                        r#type: "checkbox",
                        checked: *has_cw.read(),
                        onchange: move |_| has_cw.toggle(),
                    }
                    "⚠️ Content Warning"
                }
                label { class: "checkbox-label",
                    input {
                        r#type: "checkbox",
                        checked: *has_poll.read(),
                        onchange: move |_| has_poll.toggle(),
                    }
                    "📊 Enquete (IIAB)"
                }
            }

            // CW input
            if *has_cw.read() {
                div { class: "composer-field",
                    input {
                        class: "input-field cw-input",
                        r#type: "text",
                        placeholder: "Motivo do aviso (ex: Spoilers, Política, etc.)...",
                        value: "{cw_text}",
                        oninput: move |e| cw_text.set(e.value()),
                    }
                }
            }

            // Poll input fields
            if *has_poll.read() {
                div { class: "composer-poll-builder",
                    input {
                        class: "input-field",
                        r#type: "text",
                        placeholder: "Opção 1...",
                        value: "{poll_opt_1}",
                        oninput: move |e| poll_opt_1.set(e.value()),
                    }
                    input {
                        class: "input-field",
                        r#type: "text",
                        placeholder: "Opção 2...",
                        value: "{poll_opt_2}",
                        oninput: move |e| poll_opt_2.set(e.value()),
                    }
                }
            }

            // Submit Button
            div { class: "composer-actions",
                button {
                    class: "btn-primary",
                    disabled: !can_submit,
                    onclick: submit_handler,
                    "🚀 Publicar na IndieWeb"
                }
            }
        }
    }
}
