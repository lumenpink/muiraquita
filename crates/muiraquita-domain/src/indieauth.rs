use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error, PartialEq)]
pub enum IndieAuthError {
    #[error("URL inválida: {0}")]
    InvalidUrl(String),
    #[error("Code verifier inválido: deve possuir entre 43 e 128 caracteres")]
    InvalidCodeVerifier,
    #[error("Token sem perfil 'me'")]
    MissingMe,
    #[error("Perfil 'me' inválido ou não autorizado: {0}")]
    UnauthorizedMe(String),
    #[error("Escopo insuficiente: esperado '{0}'")]
    InsufficientScope(String),
}

/// Endpoints descobertos a partir do perfil IndieAuth do usuário
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndieAuthEndpoints {
    pub me: Url,
    pub authorization_endpoint: Url,
    pub token_endpoint: Url,
    pub micropub_endpoint: Option<Url>,
    pub microsub_endpoint: Option<Url>,
}

impl IndieAuthEndpoints {
    pub fn new(me: Url, authorization_endpoint: Url, token_endpoint: Url) -> Self {
        Self {
            me,
            authorization_endpoint,
            token_endpoint,
            micropub_endpoint: None,
            microsub_endpoint: None,
        }
    }
}

/// Gerador e validador de PKCE (RFC 7636 / IndieAuth Spec) com S256
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PkcePair {
    pub code_verifier: String,
    pub code_challenge: String,
    pub code_challenge_method: &'static str,
}

impl PkcePair {
    /// Gera um par PKCE a partir de uma semente de entropia (43 a 128 caracteres)
    pub fn from_verifier(verifier: String) -> Result<Self, IndieAuthError> {
        if verifier.len() < 43 || verifier.len() > 128 {
            return Err(IndieAuthError::InvalidCodeVerifier);
        }

        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        let challenge = URL_SAFE_NO_PAD.encode(hash);

        Ok(Self {
            code_verifier: verifier,
            code_challenge: challenge,
            code_challenge_method: "S256",
        })
    }

    /// Gera um par PKCE determinístico para testes
    pub fn mock_for_testing() -> Self {
        Self::from_verifier("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~".to_string())
            .expect("Mock verifier is valid length")
    }

    /// Validação timing-safe de code_verifier (GOST Seção 13)
    pub fn verify_verifier(&self, candidate: &str) -> bool {
        let a = self.code_verifier.as_bytes();
        let b = candidate.as_bytes();
        a.ct_eq(b).into()
    }
}

/// Token de acesso e perfil autenticado
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSession {
    pub me: Url,
    pub access_token: String,
    pub token_type: String,
    pub scope: Vec<String>,
    pub microsub_endpoint: Option<Url>,
    pub micropub_endpoint: Option<Url>,
}

impl AuthSession {
    pub fn new(
        me: Url,
        access_token: String,
        scope: Vec<String>,
        microsub_endpoint: Option<Url>,
        micropub_endpoint: Option<Url>,
    ) -> Self {
        Self {
            me,
            access_token,
            token_type: "Bearer".to_string(),
            scope,
            microsub_endpoint,
            micropub_endpoint,
        }
    }

    /// Verifica se a sessão possui determinado escopo (ex: "create", "read", "channels")
    pub fn has_scope(&self, required: &str) -> bool {
        self.scope.iter().any(|s| s.eq_ignore_ascii_case(required))
    }

    /// Comparação timing-safe de token para segurança
    pub fn verify_token(&self, candidate: &str) -> bool {
        self.access_token.as_bytes().ct_eq(candidate.as_bytes()).into()
    }
}

/// Construtor da URL de autorização IndieAuth
pub fn build_authorization_url(
    endpoint: &Url,
    me: &Url,
    client_id: &Url,
    redirect_uri: &Url,
    state: &str,
    pkce: &PkcePair,
    scopes: &[&str],
) -> Url {
    let mut url = endpoint.clone();
    let scope_str = scopes.join(" ");

    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id.as_str())
        .append_pair("redirect_uri", redirect_uri.as_str())
        .append_pair("state", state)
        .append_pair("code_challenge", &pkce.code_challenge)
        .append_pair("code_challenge_method", pkce.code_challenge_method)
        .append_pair("me", me.as_str())
        .append_pair("scope", &scope_str);

    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation_and_validation() {
        let verifier = "E9Melhoa2OwvFrGMTJguCH5rtG6j30FY9lUhv-YAm8E-test-token-12345".to_string();
        let pkce = PkcePair::from_verifier(verifier.clone()).unwrap();

        assert_eq!(pkce.code_challenge_method, "S256");
        assert!(!pkce.code_challenge.is_empty());
        assert!(!pkce.code_challenge.contains('=')); // URL_SAFE_NO_PAD

        assert!(pkce.verify_verifier(&verifier));
        assert!(!pkce.verify_verifier("candidate-errada-123456789012345678901234567890"));
    }

    #[test]
    fn test_pkce_invalid_length_rejected() {
        let short = "too-short".to_string();
        assert_eq!(
            PkcePair::from_verifier(short),
            Err(IndieAuthError::InvalidCodeVerifier)
        );
    }

    #[test]
    fn test_build_authorization_url() {
        let auth_ep = Url::parse("https://indieauth.com/auth").unwrap();
        let me = Url::parse("https://lumen.pink/").unwrap();
        let client_id = Url::parse("https://muiraquita.2lp.in/").unwrap();
        let redirect_uri = Url::parse("https://muiraquita.2lp.in/callback").unwrap();
        let pkce = PkcePair::mock_for_testing();

        let url = build_authorization_url(
            &auth_ep,
            &me,
            &client_id,
            &redirect_uri,
            "random-state-123",
            &pkce,
            &["create", "channels", "read"],
        );

        let query = url.query().unwrap();
        assert!(query.contains("response_type=code"));
        assert!(query.contains("client_id=https%3A%2F%2Fmuiraquita.2lp.in%2F"));
        assert!(query.contains("scope=create+channels+read"));
        assert!(query.contains("code_challenge_method=S256"));
    }

    #[test]
    fn test_session_scope_check() {
        let me = Url::parse("https://lumen.pink/").unwrap();
        let session = AuthSession::new(
            me,
            "secret-token-xyz".to_string(),
            vec!["create".to_string(), "channels".to_string()],
            None,
            None,
        );

        assert!(session.has_scope("create"));
        assert!(session.has_scope("channels"));
        assert!(!session.has_scope("admin"));
        assert!(session.verify_token("secret-token-xyz"));
        assert!(!session.verify_token("wrong-token"));
    }
}
