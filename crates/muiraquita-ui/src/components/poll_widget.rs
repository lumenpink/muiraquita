use dioxus::prelude::*;
use muiraquita_domain::iiab::Poll;

#[component]
pub fn PollWidget(
    poll: Poll,
    can_vote: bool,
    on_vote: EventHandler<usize>,
) -> Element {
    let total = poll.total_votes();

    rsx! {
        div { class: "poll-container",
            div { class: "poll-header",
                span { class: "poll-icon", "📊" }
                span { class: "poll-title", "Enquete Interativa ({total} votos)" }
            }
            div { class: "poll-options",
                for (idx, opt) in poll.options.iter().enumerate() {
                    {
                        let pct = poll.percentage_for_option(idx);
                        let title = opt.title.clone();
                        let votes = opt.votes;
                        rsx! {
                            div { class: "poll-option-row", key: "{idx}",
                                div { class: "poll-option-info",
                                    span { class: "poll-option-title", "{title}" }
                                    span { class: "poll-option-stats", "{votes} ({pct:.1}%)" }
                                }
                                div { class: "poll-bar-track",
                                    div {
                                        class: "poll-bar-fill",
                                        style: "width: {pct:.1}%;"
                                    }
                                }
                                if can_vote {
                                    button {
                                        class: "poll-vote-btn",
                                        onclick: move |_| on_vote.call(idx),
                                        "Votar"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
