use crate::iiab::IiabExtension;
use serde::{Deserialize, Serialize};

/// Cartão de autor microformats2 (h-card)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuthorCard {
    #[serde(rename = "type", default = "default_card_type")]
    pub card_type: String,
    pub name: Option<String>,
    pub url: Option<String>,
    pub photo: Option<String>,
}

fn default_card_type() -> String {
    "card".to_string()
}

/// Conteúdo textual e/ou HTML da entrada
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EntryContent {
    Plain(String),
    Structured {
        #[serde(default)]
        text: String,
        #[serde(default)]
        html: Option<String>,
    },
}

impl EntryContent {
    pub fn as_text(&self) -> &str {
        match self {
            EntryContent::Plain(t) => t.as_str(),
            EntryContent::Structured { text, .. } => text.as_str(),
        }
    }

    pub fn as_html(&self) -> Option<&str> {
        match self {
            EntryContent::Plain(_) => None,
            EntryContent::Structured { html, .. } => html.as_deref(),
        }
    }
}

/// Entrada normalizada JF2 (JSON Feed 2.0) conforme a especificação do Microsub W3C
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Jf2Entry {
    #[serde(rename = "type", default = "default_entry_type")]
    pub entry_type: String,
    pub uid: Option<String>,
    pub url: Option<String>,
    pub published: Option<String>,
    pub name: Option<String>,
    pub summary: Option<String>,
    pub author: Option<AuthorCard>,
    pub content: Option<EntryContent>,
    #[serde(default)]
    pub category: Vec<String>,
    #[serde(rename = "in-reply-to", default)]
    pub in_reply_to: Vec<String>,
    #[serde(rename = "like-of", default)]
    pub like_of: Vec<String>,
    #[serde(rename = "repost-of", default)]
    pub repost_of: Vec<String>,
    #[serde(default)]
    pub photo: Vec<String>,
    #[serde(rename = "_indieinabox", default)]
    pub indieinabox: Option<IiabExtension>,
}

fn default_entry_type() -> String {
    "entry".to_string()
}

impl Jf2Entry {
    pub fn display_title(&self) -> String {
        if let Some(n) = &self.name
            && !n.trim().is_empty()
        {
            return n.clone();
        }
        if let Some(s) = &self.summary
            && !s.trim().is_empty()
        {
            return s.chars().take(80).collect::<String>();
        }
        if let Some(ref c) = self.content {
            let t = c.as_text();
            let truncated: String = t.chars().take(80).collect();
            if t.chars().count() > 80 {
                return format!("{truncated}...");
            }
            return truncated;
        }
        "Sem título".to_string()
    }

    /// Retorna o texto do autor ou fallback
    pub fn author_name(&self) -> &str {
        self.author
            .as_ref()
            .and_then(|a| a.name.as_deref())
            .unwrap_or("Desconhecido")
    }

    /// Retorna a extensão IIAB ou uma padrão vazia
    pub fn iiab(&self) -> IiabExtension {
        self.indieinabox.clone().unwrap_or_default()
    }

    /// Verifica se o post possui Content Warning
    pub fn content_warning(&self) -> Option<&str> {
        self.indieinabox
            .as_ref()
            .and_then(|i| i.content_warning.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jf2_entry_parsing() {
        let json = r#"{
            "type": "entry",
            "uid": "https://example.com/post/1",
            "url": "https://example.com/post/1",
            "published": "2026-10-06T12:00:00Z",
            "author": {
                "type": "card",
                "name": "Lumen",
                "url": "https://lumen.pink"
            },
            "content": {
                "text": "Construindo o Muiraquitã com Dioxus 0.8!",
                "html": "<p>Construindo o <strong>Muiraquitã</strong> com Dioxus 0.8!</p>"
            },
            "category": ["rust", "indieweb"],
            "_indieinabox": {
                "network": "activitypub",
                "capabilities": ["like", "reply"]
            }
        }"#;

        let entry: Jf2Entry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.entry_type, "entry");
        assert_eq!(entry.author_name(), "Lumen");
        assert_eq!(entry.display_title(), "Construindo o Muiraquitã com Dioxus 0.8!");
        assert_eq!(entry.category, vec!["rust", "indieweb"]);

        let iiab = entry.iiab();
        assert!(iiab.can_like());
        assert!(iiab.can_reply());
        assert!(!iiab.can_vote());
    }

    #[test]
    fn test_jf2_entry_content_plain() {
        let json = r#"{
            "type": "entry",
            "content": "Apenas texto plano aqui"
        }"#;

        let entry: Jf2Entry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.content.unwrap().as_text(), "Apenas texto plano aqui");
    }
}
