//! Muiraquitã Mobile Suite (Android & iOS)

/// Inicializa a suíte móvel Muiraquitã com o aplicativo Dioxus 0.8
pub fn launch_mobile() {
    dioxus::launch(muiraquita_ui::App);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_module_exists() {
        assert_eq!(std::mem::size_of_val(&launch_mobile), 0);
    }
}
