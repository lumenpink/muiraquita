use async_trait::async_trait;
use muiraquita_domain::iiab::{Capability, IiabExtension, NetworkOrigin, Poll, PollOption};
use muiraquita_domain::indieauth::{AuthSession, IndieAuthEndpoints, PkcePair};
use muiraquita_domain::jf2::{AuthorCard, EntryContent, Jf2Entry};
use muiraquita_domain::micropub::MicropubDraft;
use muiraquita_domain::microsub::{
    Channel, ChannelsResponse, InteractionType, MicrosubRequest, Paging, TimelineResponse,
};
use muiraquita_domain::ports::{
    AppConfig, AuthPort, DomainError, MicropubPort, MicrosubPort, StoragePort,
};
use std::sync::{Arc, Mutex};
use url::Url;

/// Fake in-memory para testes e modo demonstração VDNKh (GOST Seção 12)
#[derive(Clone, Default)]
pub struct FakeSocialStore {
    pub channels: Arc<Mutex<Vec<Channel>>>,
    pub entries: Arc<Mutex<Vec<Jf2Entry>>>,
    pub session: Arc<Mutex<Option<AuthSession>>>,
    pub config: Arc<Mutex<AppConfig>>,
}

impl FakeSocialStore {
    /// Inicializa o repositório em memória com dados realistas do IndieInABox
    pub fn seeded() -> Self {
        let channels = vec![
            Channel::new("timeline", "Linha do Tempo"),
            Channel::new("notifications", "Notificações"),
            Channel::new("fediverse", "Fediverse (Mastodon)"),
            Channel::new("twtxt", "Twtxt Microblog"),
        ];

        let entries = vec![
            // 1. Post com Enquete nativa (IIAB Poll)
            Jf2Entry {
                entry_type: "entry".to_string(),
                uid: Some("https://lumen.pink/notes/enquete-muiraquita".to_string()),
                url: Some("https://lumen.pink/notes/enquete-muiraquita".to_string()),
                published: Some("2026-10-06T18:00:00Z".to_string()),
                name: None,
                summary: None,
                author: Some(AuthorCard {
                    card_type: "card".to_string(),
                    name: Some("Lumen Pink".to_string()),
                    url: Some("https://lumen.pink".to_string()),
                    photo: Some("https://lumen.pink/avatar.png".to_string()),
                }),
                content: Some(EntryContent::Structured {
                    text: "Qual é a sua funcionalidade favorita do ecossistema IndieInABox?".to_string(),
                    html: Some("<p>Qual é a sua funcionalidade favorita do ecossistema IndieInABox?</p>".to_string()),
                }),
                category: vec!["indieinabox".to_string(), "enquete".to_string()],
                in_reply_to: vec![],
                like_of: vec![],
                repost_of: vec![],
                photo: vec![],
                indieinabox: Some(IiabExtension {
                    network: Some(NetworkOrigin::MicropubLocal),
                    origin_server: Some("lumen.pink".to_string()),
                    capabilities: vec![Capability::Reply, Capability::Like, Capability::PollVote],
                    content_warning: None,
                    poll: Some(Poll {
                        multiple_choice: false,
                        options: vec![
                            PollOption { title: "Microsub estendido com Polls e CW".to_string(), votes: 34 },
                            PollOption { title: "Ponte com ATProto e Nostr".to_string(), votes: 19 },
                            PollOption { title: "Composição estática ultrarrápida".to_string(), votes: 27 },
                        ],
                    }),
                    reels: None,
                }),
            },
            // 2. Post com Content Warning (CW) do Fediverse
            Jf2Entry {
                entry_type: "entry".to_string(),
                uid: Some("https://mastodon.social/@amigo/987654321".to_string()),
                url: Some("https://mastodon.social/@amigo/987654321".to_string()),
                published: Some("2026-10-06T16:30:00Z".to_string()),
                name: None,
                summary: Some("Alerta de Spoiler sobre o novo framework".to_string()),
                author: Some(AuthorCard {
                    card_type: "card".to_string(),
                    name: Some("Amigo Fediverse".to_string()),
                    url: Some("https://mastodon.social/@amigo".to_string()),
                    photo: None,
                }),
                content: Some(EntryContent::Structured {
                    text: "O desfecho do artigo sobre Rust 2024 e Dioxus 0.8 mostra que a reatividade por signals supera qualquer abordagem antiga!".to_string(),
                    html: Some("<p>O desfecho do artigo sobre Rust 2024 e Dioxus 0.8 mostra que a reatividade por signals supera qualquer abordagem antiga!</p>".to_string()),
                }),
                category: vec!["rust".to_string(), "dioxus".to_string()],
                in_reply_to: vec![],
                like_of: vec![],
                repost_of: vec![],
                photo: vec![],
                indieinabox: Some(IiabExtension {
                    network: Some(NetworkOrigin::Activitypub),
                    origin_server: Some("mastodon.social".to_string()),
                    capabilities: vec![Capability::Reply, Capability::Like, Capability::Repost],
                    content_warning: Some("Spoilers sobre Rust 2024".to_string()),
                    poll: None,
                    reels: None,
                }),
            },
            // 3. Post Twtxt Microblog com hashtags
            Jf2Entry {
                entry_type: "entry".to_string(),
                uid: Some("https://txt.2lp.in/twtxt.txt#item3".to_string()),
                url: Some("https://txt.2lp.in/".to_string()),
                published: Some("2026-10-06T15:10:00Z".to_string()),
                name: None,
                summary: None,
                author: Some(AuthorCard {
                    card_type: "card".to_string(),
                    name: Some("Travesti na TI".to_string()),
                    url: Some("https://travesti.2lp.in/pt/".to_string()),
                    photo: None,
                }),
                content: Some(EntryContent::Plain(
                    "Construindo ferramentas autônomas na web aberta com #indieweb e #muiraquita! Viva o software livre.".to_string(),
                )),
                category: vec!["indieweb".to_string(), "muiraquita".to_string()],
                in_reply_to: vec![],
                like_of: vec![],
                repost_of: vec![],
                photo: vec![],
                indieinabox: Some(IiabExtension {
                    network: Some(NetworkOrigin::Twtxt),
                    origin_server: Some("txt.2lp.in".to_string()),
                    capabilities: vec![Capability::Like],
                    content_warning: None,
                    poll: None,
                    reels: None,
                }),
            },
        ];

        let default_config = AppConfig {
            default_me: Some("https://lumen.pink/".to_string()),
            client_id: "https://muiraquita.2lp.in/".to_string(),
            redirect_uri: "https://muiraquita.2lp.in/callback".to_string(),
            theme: "dark".to_string(),
            auto_refresh_seconds: 60,
        };

        Self {
            channels: Arc::new(Mutex::new(channels)),
            entries: Arc::new(Mutex::new(entries)),
            session: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(default_config)),
        }
    }
}

#[async_trait]
impl AuthPort for FakeSocialStore {
    async fn discover_endpoints(&self, me: &Url) -> Result<IndieAuthEndpoints, DomainError> {
        Ok(IndieAuthEndpoints {
            me: me.clone(),
            authorization_endpoint: Url::parse("https://indieauth.com/auth").unwrap(),
            token_endpoint: Url::parse("https://tokens.indieauth.com/token").unwrap(),
            micropub_endpoint: Some(Url::parse("https://lumen.pink/micropub").unwrap()),
            microsub_endpoint: Some(Url::parse("https://lumen.pink/microsub").unwrap()),
        })
    }

    async fn exchange_code(
        &self,
        _token_endpoint: &Url,
        _code: &str,
        me: &Url,
        _redirect_uri: &Url,
        _client_id: &Url,
        _pkce: &PkcePair,
    ) -> Result<AuthSession, DomainError> {
        let session = AuthSession::new(
            me.clone(),
            "fake-access-token-12345".to_string(),
            vec!["create".to_string(), "channels".to_string(), "read".to_string()],
            Some(Url::parse("https://lumen.pink/microsub").unwrap()),
            Some(Url::parse("https://lumen.pink/micropub").unwrap()),
        );

        let mut lock = self.session.lock().unwrap();
        *lock = Some(session.clone());
        Ok(session)
    }
}

#[async_trait]
impl MicrosubPort for FakeSocialStore {
    async fn get_channels(
        &self,
        _endpoint: &Url,
        _token: &str,
    ) -> Result<ChannelsResponse, DomainError> {
        let channels = self.channels.lock().unwrap().clone();
        Ok(ChannelsResponse { channels })
    }

    async fn get_timeline(
        &self,
        _endpoint: &Url,
        _token: &str,
        _req: &MicrosubRequest,
    ) -> Result<TimelineResponse, DomainError> {
        let items = self.entries.lock().unwrap().clone();
        Ok(TimelineResponse {
            items,
            paging: Some(Paging {
                after: Some("fake-cursor-after".to_string()),
                before: None,
            }),
        })
    }

    async fn send_action(
        &self,
        _endpoint: &Url,
        _token: &str,
        req: &MicrosubRequest,
    ) -> Result<(), DomainError> {
        if let MicrosubRequest::Interact {
            interaction_type: InteractionType::PollVote,
            url,
            poll_option: Some(opt_idx),
            ..
        } = req
        {
            let mut lock = self.entries.lock().unwrap();
            if let Some(entry) = lock.iter_mut().find(|e| e.url.as_deref() == Some(url))
                && let Some(ref mut ext) = entry.indieinabox
                && let Some(ref mut poll) = ext.poll
                && let Some(opt) = poll.options.get_mut(*opt_idx)
            {
                opt.votes += 1;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl MicropubPort for FakeSocialStore {
    async fn create_post(
        &self,
        _endpoint: &Url,
        _token: &str,
        draft: &MicropubDraft,
    ) -> Result<Option<String>, DomainError> {
        let new_url = format!("https://lumen.pink/notes/{}", chrono::Utc::now().timestamp());
        let new_entry = Jf2Entry {
            entry_type: "entry".to_string(),
            uid: Some(new_url.clone()),
            url: Some(new_url.clone()),
            published: Some(chrono::Utc::now().to_rfc3339()),
            name: draft.name.clone(),
            summary: None,
            author: Some(AuthorCard {
                card_type: "card".to_string(),
                name: Some("Lumen Pink".to_string()),
                url: Some("https://lumen.pink".to_string()),
                photo: None,
            }),
            content: Some(EntryContent::Plain(draft.content.clone())),
            category: draft.categories.clone(),
            in_reply_to: draft.in_reply_to.clone().into_iter().collect(),
            like_of: draft.like_of.clone().into_iter().collect(),
            repost_of: draft.repost_of.clone().into_iter().collect(),
            photo: draft.photos.clone(),
            indieinabox: Some(IiabExtension {
                network: Some(NetworkOrigin::MicropubLocal),
                origin_server: Some("lumen.pink".to_string()),
                capabilities: vec![Capability::Like, Capability::Reply],
                content_warning: draft.content_warning.clone(),
                poll: draft.poll.clone(),
                reels: None,
            }),
        };

        let mut lock = self.entries.lock().unwrap();
        lock.insert(0, new_entry);

        Ok(Some(new_url))
    }
}

#[async_trait]
impl StoragePort for FakeSocialStore {
    async fn load_session(&self) -> Result<Option<AuthSession>, DomainError> {
        Ok(self.session.lock().unwrap().clone())
    }

    async fn save_session(&self, session: &AuthSession) -> Result<(), DomainError> {
        *self.session.lock().unwrap() = Some(session.clone());
        Ok(())
    }

    async fn clear_session(&self) -> Result<(), DomainError> {
        *self.session.lock().unwrap() = None;
        Ok(())
    }

    async fn load_config(&self) -> Result<AppConfig, DomainError> {
        Ok(self.config.lock().unwrap().clone())
    }

    async fn save_config(&self, config: &AppConfig) -> Result<(), DomainError> {
        *self.config.lock().unwrap() = config.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_timeline_and_poll_vote() {
        let store = FakeSocialStore::seeded();
        let dummy_url = Url::parse("https://lumen.pink/microsub").unwrap();

        // 1. Get channels
        let channels = store.get_channels(&dummy_url, "token").await.unwrap();
        assert_eq!(channels.channels.len(), 4);

        // 2. Get timeline
        let req = MicrosubRequest::Timeline {
            channel: "timeline".to_string(),
            after: None,
            before: None,
            limit: None,
        };
        let timeline = store.get_timeline(&dummy_url, "token", &req).await.unwrap();
        assert_eq!(timeline.items.len(), 3);

        let poll_entry = &timeline.items[0];
        let poll = poll_entry.iiab().poll.unwrap();
        let votes_before = poll.options[0].votes;

        // 3. Vote on option 0
        let vote_req = MicrosubRequest::Interact {
            interaction_type: InteractionType::PollVote,
            url: poll_entry.url.clone().unwrap(),
            content: None,
            poll_option: Some(0),
        };
        store.send_action(&dummy_url, "token", &vote_req).await.unwrap();

        // 4. Verify vote count incremented in-memory
        let timeline_after = store.get_timeline(&dummy_url, "token", &req).await.unwrap();
        let poll_after = timeline_after.items[0].iiab().poll.unwrap();
        assert_eq!(poll_after.options[0].votes, votes_before + 1);
    }

    #[tokio::test]
    async fn test_fake_micropub_post_creation() {
        let store = FakeSocialStore::seeded();
        let dummy_url = Url::parse("https://lumen.pink/micropub").unwrap();

        let draft = MicropubDraft::new_note("Testando nova nota com #muiraquita!");
        let created_url = store.create_post(&dummy_url, "token", &draft).await.unwrap();
        assert!(created_url.is_some());

        let req = MicrosubRequest::Timeline {
            channel: "timeline".to_string(),
            after: None,
            before: None,
            limit: None,
        };
        let timeline = store.get_timeline(&dummy_url, "token", &req).await.unwrap();
        assert_eq!(timeline.items.len(), 4);
        assert_eq!(timeline.items[0].display_title(), "Testando nova nota com #muiraquita!");
    }
}
