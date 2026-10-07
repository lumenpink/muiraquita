use dioxus::prelude::*;
use muiraquita_domain::indieauth::AuthSession;
use muiraquita_domain::jf2::Jf2Entry;
use muiraquita_domain::microsub::Channel;
use muiraquita_domain::ports::AppConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Timeline,
    Channels,
    Compose,
    Settings,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub active_tab: Signal<ActiveTab>,
    pub selected_channel: Signal<String>,
    pub channels: Signal<Vec<Channel>>,
    pub entries: Signal<Vec<Jf2Entry>>,
    pub session: Signal<Option<AuthSession>>,
    pub config: Signal<AppConfig>,
    pub is_loading: Signal<bool>,
    pub status_msg: Signal<Option<String>>,
    pub auth_modal_open: Signal<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_tab: Signal::new(ActiveTab::Timeline),
            selected_channel: Signal::new("timeline".to_string()),
            channels: Signal::new(vec![]),
            entries: Signal::new(vec![]),
            session: Signal::new(None),
            config: Signal::new(AppConfig::default()),
            is_loading: Signal::new(false),
            status_msg: Signal::new(None),
            auth_modal_open: Signal::new(false),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
