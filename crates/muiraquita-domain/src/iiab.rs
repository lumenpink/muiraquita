use serde::{Deserialize, Serialize};

/// Protocolos e redes de origem suportados nativamente pelo IndieInABox
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkOrigin {
    Activitypub,
    Twtxt,
    Atproto,
    Nostr,
    Rss,
    MicropubLocal,
    #[serde(untagged)]
    Custom(String),
}

impl NetworkOrigin {
    pub fn display_badge(&self) -> &'static str {
        match self {
            NetworkOrigin::Activitypub => "Fediverse",
            NetworkOrigin::Twtxt => "twtxt",
            NetworkOrigin::Atproto => "Bluesky",
            NetworkOrigin::Nostr => "Nostr",
            NetworkOrigin::Rss => "RSS/Atom",
            NetworkOrigin::MicropubLocal => "IndieWeb",
            NetworkOrigin::Custom(_) => "Custom",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            NetworkOrigin::Activitypub => "🪐",
            NetworkOrigin::Twtxt => "📜",
            NetworkOrigin::Atproto => "🦋",
            NetworkOrigin::Nostr => "⚡",
            NetworkOrigin::Rss => "📡",
            NetworkOrigin::MicropubLocal => "🐸",
            NetworkOrigin::Custom(_) => "🌐",
        }
    }
}

/// Ações que este post específico permite que o cliente execute
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Reply,
    Like,
    Repost,
    PollVote,
}

/// Opção individual de enquete
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PollOption {
    pub title: String,
    #[serde(default)]
    pub votes: u64,
}

/// Estrutura de Enquete nativa (compatível com ActivityPub e Microsub)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Poll {
    #[serde(default)]
    pub multiple_choice: bool,
    pub options: Vec<PollOption>,
}

impl Poll {
    pub fn new(options: Vec<PollOption>, multiple_choice: bool) -> Self {
        Self {
            multiple_choice,
            options,
        }
    }

    pub fn total_votes(&self) -> u64 {
        self.options.iter().map(|o| o.votes).sum()
    }

    pub fn percentage_for_option(&self, idx: usize) -> f64 {
        let total = self.total_votes();
        if total == 0 {
            0.0
        } else if let Some(opt) = self.options.get(idx) {
            (opt.votes as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Metadados de vídeo curto em loop (Pixelfed/PeerTube)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReelsMetadata {
    pub video_url: String,
    pub duration_seconds: Option<f64>,
    pub loop_count: Option<u32>,
}

/// Namespace oficial `_indieinabox` anexado a cada post JF2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IiabExtension {
    #[serde(default)]
    pub network: Option<NetworkOrigin>,
    #[serde(default)]
    pub origin_server: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    #[serde(default)]
    pub content_warning: Option<String>,
    #[serde(default)]
    pub poll: Option<Poll>,
    #[serde(default)]
    pub reels: Option<ReelsMetadata>,
}

impl IiabExtension {
    pub fn can_reply(&self) -> bool {
        self.capabilities.contains(&Capability::Reply)
    }

    pub fn can_like(&self) -> bool {
        self.capabilities.contains(&Capability::Like)
    }

    pub fn can_repost(&self) -> bool {
        self.capabilities.contains(&Capability::Repost)
    }

    pub fn can_vote(&self) -> bool {
        self.capabilities.contains(&Capability::PollVote) && self.poll.is_some()
    }
}

/// Extrator automático de hashtags no corpo do texto (ex: #indieweb -> "indieweb")
pub fn extract_hashtags(content: &str) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    for word in content.split_whitespace() {
        if word.starts_with('#') && word.len() > 1 {
            let clean = word
                .trim_start_matches('#')
                .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
            if !clean.is_empty() && !tags.iter().any(|t: &String| t.eq_ignore_ascii_case(clean)) {
                tags.push(clean.to_lowercase());
            }
        }
    }
    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iiab_extension_deserialization() {
        let json = r#"{
            "network": "activitypub",
            "origin_server": "mastodon.social",
            "capabilities": ["reply", "like", "repost", "poll_vote"],
            "content_warning": "Spoiler da Série",
            "poll": {
                "multiple_choice": false,
                "options": [
                    { "title": "Gostei", "votes": 42 },
                    { "title": "Não gostei", "votes": 14 }
                ]
            }
        }"#;

        let ext: IiabExtension = serde_json::from_str(json).unwrap();
        assert_eq!(ext.network, Some(NetworkOrigin::Activitypub));
        assert_eq!(ext.origin_server, Some("mastodon.social".to_string()));
        assert_eq!(ext.content_warning, Some("Spoiler da Série".to_string()));
        assert!(ext.can_vote());
        assert!(ext.can_like());

        let poll = ext.poll.unwrap();
        assert_eq!(poll.total_votes(), 56);
        assert!((poll.percentage_for_option(0) - 75.0).abs() < 0.01);
        assert!((poll.percentage_for_option(1) - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_extract_hashtags() {
        let text = "Olá mundo! Testando o #indieweb e #indieinabox no #Rust2024! Também #indieweb duplicado.";
        let tags = extract_hashtags(text);
        assert_eq!(tags, vec!["indieweb", "indieinabox", "rust2024"]);
    }
}
