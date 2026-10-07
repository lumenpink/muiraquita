use crate::jf2::Jf2Entry;
use serde::{Deserialize, Serialize};

/// Canal Microsub (ex: "notifications", "timeline", "read-later")
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Channel {
    pub uid: String,
    pub name: String,
    #[serde(default)]
    pub unread: Option<u32>,
}

impl Channel {
    pub fn new(uid: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            uid: uid.into(),
            name: name.into(),
            unread: None,
        }
    }
}

/// Metadados de paginação de timeline
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Paging {
    pub after: Option<String>,
    pub before: Option<String>,
}

/// Resposta padrão de consulta de timeline do Microsub
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineResponse {
    pub items: Vec<Jf2Entry>,
    #[serde(default)]
    pub paging: Option<Paging>,
}

/// Resposta da lista de canais
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelsResponse {
    pub channels: Vec<Channel>,
}

/// Ações que o cliente Microsub envia para o servidor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum MicrosubRequest {
    Channels,
    Timeline {
        channel: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        after: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        before: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
    },
    Follow {
        channel: String,
        url: String,
    },
    Unfollow {
        channel: String,
        url: String,
    },
    Mute {
        channel: String,
        url: String,
    },
    Block {
        channel: String,
        url: String,
    },
    /// Interação social com um post
    Interact {
        #[serde(rename = "type")]
        interaction_type: InteractionType,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        /// Extensão IIAB para votação em enquetes
        #[serde(skip_serializing_if = "Option::is_none")]
        poll_option: Option<usize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionType {
    Like,
    Repost,
    Reply,
    PollVote,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_response_deserialization() {
        let json = r#"{
            "items": [
                {
                    "type": "entry",
                    "uid": "1",
                    "content": "Post um"
                },
                {
                    "type": "entry",
                    "uid": "2",
                    "content": "Post dois"
                }
            ],
            "paging": {
                "after": "cursor-after-123",
                "before": "cursor-before-456"
            }
        }"#;

        let res: TimelineResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.items.len(), 2);
        let paging = res.paging.unwrap();
        assert_eq!(paging.after.as_deref(), Some("cursor-after-123"));
        assert_eq!(paging.before.as_deref(), Some("cursor-before-456"));
    }

    #[test]
    fn test_microsub_poll_vote_interaction() {
        let req = MicrosubRequest::Interact {
            interaction_type: InteractionType::PollVote,
            url: "https://mastodon.social/@user/123".to_string(),
            content: None,
            poll_option: Some(1),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"action\":\"interact\""));
        assert!(json.contains("\"type\":\"poll_vote\""));
        assert!(json.contains("\"poll_option\":1"));
    }
}
