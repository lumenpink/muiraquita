use crate::indieauth::{AuthSession, IndieAuthEndpoints, PkcePair};
use crate::micropub::MicropubDraft;
use crate::microsub::{ChannelsResponse, MicrosubRequest, TimelineResponse};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error, PartialEq)]
pub enum DomainError {
    #[error("Erro de rede / transporte: {0}")]
    Network(String),
    #[error("Erro de autenticação: {0}")]
    Auth(String),
    #[error("Erro de persistência: {0}")]
    Storage(String),
    #[error("Protocolo inválido ou payload inesperado: {0}")]
    Protocol(String),
    #[error("Endpoint não configurado ou ausente")]
    MissingEndpoint,
}

/// Port para descoberta IndieAuth e troca de tokens
#[async_trait::async_trait]
pub trait AuthPort: Send + Sync {
    /// Descobre endpoints a partir da URL do perfil do usuário
    async fn discover_endpoints(&self, me: &Url) -> Result<IndieAuthEndpoints, DomainError>;

    /// Troca o código de autorização por um token de acesso
    async fn exchange_code(
        &self,
        token_endpoint: &Url,
        code: &str,
        me: &Url,
        redirect_uri: &Url,
        client_id: &Url,
        pkce: &PkcePair,
    ) -> Result<AuthSession, DomainError>;
}

/// Port para consultas e ações Microsub
#[async_trait::async_trait]
pub trait MicrosubPort: Send + Sync {
    /// Consulta a lista de canais
    async fn get_channels(
        &self,
        endpoint: &Url,
        token: &str,
    ) -> Result<ChannelsResponse, DomainError>;

    /// Consulta a timeline de um canal
    async fn get_timeline(
        &self,
        endpoint: &Url,
        token: &str,
        req: &MicrosubRequest,
    ) -> Result<TimelineResponse, DomainError>;

    /// Envia uma ação (like, repost, reply, poll_vote)
    async fn send_action(
        &self,
        endpoint: &Url,
        token: &str,
        req: &MicrosubRequest,
    ) -> Result<(), DomainError>;
}

/// Port para publicação Micropub
#[async_trait::async_trait]
pub trait MicropubPort: Send + Sync {
    /// Cria uma nova entrada via Micropub
    async fn create_post(
        &self,
        endpoint: &Url,
        token: &str,
        draft: &MicropubDraft,
    ) -> Result<Option<String>, DomainError>;
}

/// Configurações gerais do usuário (conformidade XDG)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub default_me: Option<String>,
    pub client_id: String,
    pub redirect_uri: String,
    pub theme: String,
    pub auto_refresh_seconds: u32,
}

/// Port para persistência local de sessão e configurações
#[async_trait::async_trait]
pub trait StoragePort: Send + Sync {
    async fn load_session(&self) -> Result<Option<AuthSession>, DomainError>;
    async fn save_session(&self, session: &AuthSession) -> Result<(), DomainError>;
    async fn clear_session(&self) -> Result<(), DomainError>;
    async fn load_config(&self) -> Result<AppConfig, DomainError>;
    async fn save_config(&self, config: &AppConfig) -> Result<(), DomainError>;
}
