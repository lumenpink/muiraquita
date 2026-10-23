use async_trait::async_trait;
use muiraquita_domain::indieauth::AuthSession;
use muiraquita_domain::ports::{AppConfig, DomainError, StoragePort};
use std::path::PathBuf;

/// Adaptador de persistência aderente à especificação XDG Base Directory (GOST Seção 13)
pub struct XdgStorage {
    config_dir: PathBuf,
    state_dir: PathBuf,
}

impl XdgStorage {
    pub fn new() -> Self {
        let config_home = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".config")
            });

        let state_home = std::env::var("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".local").join("state")
            });

        Self {
            config_dir: config_home.join("muiraquita"),
            state_dir: state_home.join("muiraquita"),
        }
    }

    pub fn with_custom_dirs(config_dir: PathBuf, state_dir: PathBuf) -> Self {
        Self {
            config_dir,
            state_dir,
        }
    }

    fn config_path(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    fn session_path(&self) -> PathBuf {
        self.state_dir.join("session.json")
    }
}

impl Default for XdgStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StoragePort for XdgStorage {
    async fn load_session(&self) -> Result<Option<AuthSession>, DomainError> {
        let path = self.session_path();
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| DomainError::Storage(format!("Falha ao ler sessão: {e}")))?;
        let session = serde_json::from_str(&content)
            .map_err(|e| DomainError::Storage(format!("JSON de sessão inválido: {e}")))?;
        Ok(Some(session))
    }

    async fn save_session(&self, session: &AuthSession) -> Result<(), DomainError> {
        let path = self.session_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DomainError::Storage(format!("Falha ao criar diretório XDG State: {e}"))
            })?;
        }
        let content = serde_json::to_string_pretty(session)
            .map_err(|e| DomainError::Storage(format!("Falha ao serializar sessão: {e}")))?;
        std::fs::write(&path, content)
            .map_err(|e| DomainError::Storage(format!("Falha ao gravar sessão XDG: {e}")))?;
        Ok(())
    }

    async fn clear_session(&self) -> Result<(), DomainError> {
        let path = self.session_path();
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        Ok(())
    }

    async fn load_config(&self) -> Result<AppConfig, DomainError> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(AppConfig {
                default_me: None,
                client_id: "https://muiraquita.2lp.in/".to_string(),
                redirect_uri: "https://muiraquita.2lp.in/callback".to_string(),
                theme: "dark".to_string(),
                auto_refresh_seconds: 60,
            });
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| DomainError::Storage(format!("Falha ao ler config: {e}")))?;
        let config = serde_json::from_str(&content)
            .map_err(|e| DomainError::Storage(format!("JSON de config inválido: {e}")))?;
        Ok(config)
    }

    async fn save_config(&self, config: &AppConfig) -> Result<(), DomainError> {
        let path = self.config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DomainError::Storage(format!("Falha ao criar diretório XDG Config: {e}"))
            })?;
        }
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| DomainError::Storage(format!("Falha ao serializar config: {e}")))?;
        std::fs::write(&path, content)
            .map_err(|e| DomainError::Storage(format!("Falha ao gravar config XDG: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[tokio::test]
    async fn test_xdg_storage_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("muiraquita_test_{}", std::process::id()));
        let config_dir = temp_dir.join("config");
        let state_dir = temp_dir.join("state");

        let storage = XdgStorage::with_custom_dirs(config_dir.clone(), state_dir.clone());

        // 1. Session is none initially
        assert!(storage.load_session().await.unwrap().is_none());

        // 2. Save session
        let me = Url::parse("https://lumen.pink/").unwrap();
        let session = AuthSession::new(
            me,
            "secret-token".to_string(),
            vec!["create".to_string()],
            None,
            None,
        );
        storage.save_session(&session).await.unwrap();

        // 3. Load session
        let loaded = storage.load_session().await.unwrap().unwrap();
        assert_eq!(loaded.access_token, "secret-token");

        // 4. Clear session
        storage.clear_session().await.unwrap();
        assert!(storage.load_session().await.unwrap().is_none());

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
