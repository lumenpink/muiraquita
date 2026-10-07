use crate::iiab::{extract_hashtags, Poll};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tipos de postagem suportados pelo Micropub W3C
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PostKind {
    Note,
    Article,
    Reply,
    Like,
    Repost,
    Bookmark,
}

impl PostKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            PostKind::Note => "Nota",
            PostKind::Article => "Artigo",
            PostKind::Reply => "Resposta",
            PostKind::Like => "Curtida",
            PostKind::Repost => "Repost",
            PostKind::Bookmark => "Marcador",
        }
    }
}

/// Estrutura de autoria e envio para o endpoint Micropub
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MicropubDraft {
    pub kind: Option<PostKind>,
    pub content: String,
    pub name: Option<String>,
    pub in_reply_to: Option<String>,
    pub like_of: Option<String>,
    pub repost_of: Option<String>,
    pub categories: Vec<String>,
    pub photos: Vec<String>,
    pub mp_syndicate_to: Vec<String>,
    /// Extensão IIAB para Content Warning
    pub content_warning: Option<String>,
    /// Extensão IIAB para Enquetes
    pub poll: Option<Poll>,
}

impl MicropubDraft {
    pub fn new_note(content: impl Into<String>) -> Self {
        let content_str = content.into();
        let auto_tags = extract_hashtags(&content_str);
        Self {
            kind: Some(PostKind::Note),
            content: content_str,
            categories: auto_tags,
            ..Default::default()
        }
    }

    pub fn new_article(title: impl Into<String>, content: impl Into<String>) -> Self {
        let content_str = content.into();
        let auto_tags = extract_hashtags(&content_str);
        Self {
            kind: Some(PostKind::Article),
            name: Some(title.into()),
            content: content_str,
            categories: auto_tags,
            ..Default::default()
        }
    }

    pub fn new_reply(in_reply_to: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            kind: Some(PostKind::Reply),
            in_reply_to: Some(in_reply_to.into()),
            content: content.into(),
            ..Default::default()
        }
    }

    /// Serializa no formato canônico Micropub JSON (W3C Recommendation)
    pub fn to_json_payload(&self) -> serde_json::Value {
        let mut props: HashMap<&str, serde_json::Value> = HashMap::new();

        props.insert("content", serde_json::json!([self.content]));

        if let Some(name) = &self.name
            && !name.trim().is_empty()
        {
            props.insert("name", serde_json::json!([name]));
        }

        if let Some(ref reply) = self.in_reply_to {
            props.insert("in-reply-to", serde_json::json!([reply]));
        }

        if let Some(ref like) = self.like_of {
            props.insert("like-of", serde_json::json!([like]));
        }

        if let Some(ref repost) = self.repost_of {
            props.insert("repost-of", serde_json::json!([repost]));
        }

        if !self.categories.is_empty() {
            props.insert("category", serde_json::json!(self.categories));
        }

        if !self.photos.is_empty() {
            props.insert("photo", serde_json::json!(self.photos));
        }

        if !self.mp_syndicate_to.is_empty() {
            props.insert("mp-syndicate-to", serde_json::json!(self.mp_syndicate_to));
        }

        // Extensões IIAB anexadas
        if let Some(ref cw) = self.content_warning {
            props.insert("_indieinabox_cw", serde_json::json!([cw]));
        }

        if let Some(ref poll) = self.poll {
            props.insert("_indieinabox_poll", serde_json::json!(poll));
        }

        serde_json::json!({
            "type": ["h-entry"],
            "properties": props
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_micropub_note_json_payload() {
        let draft = MicropubDraft::new_note("Olá IndieWeb com #muiraquita!");
        let payload = draft.to_json_payload();

        let obj = payload.as_object().unwrap();
        assert_eq!(obj.get("type").unwrap(), &serde_json::json!(["h-entry"]));

        let props = obj.get("properties").unwrap().as_object().unwrap();
        assert_eq!(
            props.get("content").unwrap(),
            &serde_json::json!(["Olá IndieWeb com #muiraquita!"])
        );
        assert_eq!(
            props.get("category").unwrap(),
            &serde_json::json!(["muiraquita"])
        );
    }

    #[test]
    fn test_micropub_article_with_cw() {
        let mut draft = MicropubDraft::new_article("Meu Artigo", "Conteúdo com spoilers");
        draft.content_warning = Some("Spoilers de Filosofia".to_string());

        let payload = draft.to_json_payload();
        let props = payload["properties"].as_object().unwrap();
        assert_eq!(props.get("name").unwrap(), &serde_json::json!(["Meu Artigo"]));
        assert_eq!(
            props.get("_indieinabox_cw").unwrap(),
            &serde_json::json!(["Spoilers de Filosofia"])
        );
    }
}
