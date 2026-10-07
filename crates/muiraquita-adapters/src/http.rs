use async_trait::async_trait;
use muiraquita_domain::indieauth::{AuthSession, IndieAuthEndpoints, PkcePair};
use muiraquita_domain::microsub::{ChannelsResponse, MicrosubRequest, TimelineResponse};
use muiraquita_domain::micropub::MicropubDraft;
use muiraquita_domain::ports::{AuthPort, DomainError, MicropubPort, MicrosubPort};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use url::Url;

/// Adaptador HTTP universal (reqwest com TLS moderno)
#[derive(Clone)]
pub struct HttpAdapter {
    client: Client,
}

impl HttpAdapter {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

impl Default for HttpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Extrai URLs de tags `<link rel="..." href="...">` em HTML simples sem dependências pesadas
pub fn parse_link_rel(html: &str, target_rel: &str, base_url: &Url) -> Option<Url> {
    let lower = html.to_lowercase();
    let rel_pattern = format!("rel=\"{target_rel}\"");
    let rel_pattern_space = format!("rel=\"{target_rel} ");
    let rel_pattern_space2 = format!(" {target_rel}\"");

    let mut start_idx = 0;
    while let Some(idx) = lower[start_idx..].find("<link") {
        let tag_start = start_idx + idx;
        let tag_end = if let Some(end) = lower[tag_start..].find('>') {
            tag_start + end
        } else {
            break;
        };
        let tag = &html[tag_start..=tag_end];
        let tag_lower = tag.to_lowercase();

        let has_rel = tag_lower.contains(&rel_pattern)
            || tag_lower.contains(&rel_pattern_space)
            || tag_lower.contains(&rel_pattern_space2);

        if has_rel
            && let Some(href_idx) = tag_lower.find("href=\"")
        {
            let after = &tag[href_idx + 6..];
            if let Some(close_idx) = after.find('"')
                && let Ok(resolved) = base_url.join(&after[..close_idx])
            {
                return Some(resolved);
            }
        }
        start_idx = tag_end + 1;
    }
    None
}

#[async_trait]
impl AuthPort for HttpAdapter {
    async fn discover_endpoints(&self, me: &Url) -> Result<IndieAuthEndpoints, DomainError> {
        let res = self
            .client
            .get(me.as_str())
            .send()
            .await
            .map_err(|e| DomainError::Network(format!("Erro ao acessar perfil: {e}")))?;

        let html = res
            .text()
            .await
            .map_err(|e| DomainError::Network(format!("Erro ao ler corpo do perfil: {e}")))?;

        let auth_ep = parse_link_rel(&html, "authorization_endpoint", me)
            .ok_or_else(|| DomainError::Auth("authorization_endpoint não encontrado no perfil".to_string()))?;

        let token_ep = parse_link_rel(&html, "token_endpoint", me)
            .ok_or_else(|| DomainError::Auth("token_endpoint não encontrado no perfil".to_string()))?;

        let micropub_ep = parse_link_rel(&html, "micropub", me);
        let microsub_ep = parse_link_rel(&html, "microsub", me);

        Ok(IndieAuthEndpoints {
            me: me.clone(),
            authorization_endpoint: auth_ep,
            token_endpoint: token_ep,
            micropub_endpoint: micropub_ep,
            microsub_endpoint: microsub_ep,
        })
    }

    async fn exchange_code(
        &self,
        token_endpoint: &Url,
        code: &str,
        me: &Url,
        redirect_uri: &Url,
        client_id: &Url,
        pkce: &PkcePair,
    ) -> Result<AuthSession, DomainError> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", client_id.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code_verifier", &pkce.code_verifier),
            ("me", me.as_str()),
        ];

        let res = self
            .client
            .post(token_endpoint.as_str())
            .form(&params)
            .send()
            .await
            .map_err(|e| DomainError::Network(format!("Erro na troca de código no token endpoint: {e}")))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(DomainError::Auth(format!("Falha na autorização: {err_text}")));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| DomainError::Protocol(format!("Resposta JSON de token inválida: {e}")))?;

        let access_token = json["access_token"]
            .as_str()
            .ok_or_else(|| DomainError::Auth("access_token ausente na resposta".to_string()))?
            .to_string();

        let returned_me = if let Some(me_str) = json["me"].as_str() {
            Url::parse(me_str).map_err(|e| DomainError::Auth(format!("URL 'me' retornada inválida: {e}")))?
        } else {
            me.clone()
        };

        let scope = if let Some(scope_str) = json["scope"].as_str() {
            scope_str.split_whitespace().map(|s| s.to_string()).collect()
        } else {
            vec![]
        };

        let microsub_endpoint = json["microsub_endpoint"]
            .as_str()
            .and_then(|u| Url::parse(u).ok());

        let micropub_endpoint = json["micropub_endpoint"]
            .as_str()
            .and_then(|u| Url::parse(u).ok());

        Ok(AuthSession::new(
            returned_me,
            access_token,
            scope,
            microsub_endpoint,
            micropub_endpoint,
        ))
    }
}

#[async_trait]
impl MicrosubPort for HttpAdapter {
    async fn get_channels(
        &self,
        endpoint: &Url,
        token: &str,
    ) -> Result<ChannelsResponse, DomainError> {
        let mut url = endpoint.clone();
        url.query_pairs_mut().append_pair("action", "channels");

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );

        let res = self
            .client
            .get(url.as_str())
            .headers(headers)
            .send()
            .await
            .map_err(|e| DomainError::Network(format!("Erro ao buscar canais Microsub: {e}")))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(DomainError::Protocol(format!("Erro do servidor Microsub: {err_text}")));
        }

        let channels: ChannelsResponse = res
            .json()
            .await
            .map_err(|e| DomainError::Protocol(format!("JSON de canais inválido: {e}")))?;

        Ok(channels)
    }

    async fn get_timeline(
        &self,
        endpoint: &Url,
        token: &str,
        req: &MicrosubRequest,
    ) -> Result<TimelineResponse, DomainError> {
        let mut url = endpoint.clone();
        if let MicrosubRequest::Timeline {
            channel,
            after,
            before,
            limit,
        } = req
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("action", "timeline");
            pairs.append_pair("channel", channel);
            if let Some(a) = after {
                pairs.append_pair("after", a);
            }
            if let Some(b) = before {
                pairs.append_pair("before", b);
            }
            if let Some(l) = limit {
                pairs.append_pair("limit", &l.to_string());
            }
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );

        let res = self
            .client
            .get(url.as_str())
            .headers(headers)
            .send()
            .await
            .map_err(|e| DomainError::Network(format!("Erro ao buscar timeline Microsub: {e}")))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(DomainError::Protocol(format!("Erro do servidor Microsub: {err_text}")));
        }

        let timeline: TimelineResponse = res
            .json()
            .await
            .map_err(|e| DomainError::Protocol(format!("JSON de timeline inválido: {e}")))?;

        Ok(timeline)
    }

    async fn send_action(
        &self,
        endpoint: &Url,
        token: &str,
        req: &MicrosubRequest,
    ) -> Result<(), DomainError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );

        let res = self
            .client
            .post(endpoint.as_str())
            .headers(headers)
            .json(req)
            .send()
            .await
            .map_err(|e| DomainError::Network(format!("Erro ao enviar ação Microsub: {e}")))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(DomainError::Protocol(format!("Ação rejeitada pelo servidor: {err_text}")));
        }

        Ok(())
    }
}

#[async_trait]
impl MicropubPort for HttpAdapter {
    async fn create_post(
        &self,
        endpoint: &Url,
        token: &str,
        draft: &MicropubDraft,
    ) -> Result<Option<String>, DomainError> {
        let payload = draft.to_json_payload();

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let res = self
            .client
            .post(endpoint.as_str())
            .headers(headers)
            .json(&payload)
            .send()
            .await
            .map_err(|e| DomainError::Network(format!("Erro ao enviar post Micropub: {e}")))?;

        if !res.status().is_success() && res.status().as_u16() != 201 && res.status().as_u16() != 202 {
            let err_text = res.text().await.unwrap_or_default();
            return Err(DomainError::Protocol(format!("Criação Micropub rejeitada: {err_text}")));
        }

        let location = res
            .headers()
            .get("Location")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        Ok(location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_link_rel() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <link rel="authorization_endpoint" href="https://indieauth.com/auth">
                <link rel="token_endpoint" href="/token">
                <link rel="micropub" href="https://lumen.pink/micropub">
            </head>
            </html>
        "#;
        let base = Url::parse("https://lumen.pink/").unwrap();

        let auth_ep = parse_link_rel(html, "authorization_endpoint", &base).unwrap();
        assert_eq!(auth_ep.as_str(), "https://indieauth.com/auth");

        let token_ep = parse_link_rel(html, "token_endpoint", &base).unwrap();
        assert_eq!(token_ep.as_str(), "https://lumen.pink/token");

        let micropub_ep = parse_link_rel(html, "micropub", &base).unwrap();
        assert_eq!(micropub_ep.as_str(), "https://lumen.pink/micropub");

        assert!(parse_link_rel(html, "microsub", &base).is_none());
    }
}
