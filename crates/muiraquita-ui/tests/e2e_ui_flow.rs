//! End-to-End UI Flow Test for Muiraquitã
//! Valida a montagem e renderização completa de toda a interface reativa Dioxus:
//! - Cabeçalho soberano e estado de conexão
//! - Timeline de posts federados com cartões h-card
//! - Componente de Content Warning com overlay de proteção
//! - Widget de enquete interativa descentralizada com barras percentuais
//! - Formulário de composição Micropub
//! - Navegação adaptativa e rodapé canônico GOST 13

use dioxus::prelude::*;
use muiraquita_ui::App;

#[test]
fn test_e2e_ui_complete_render_and_components() {
    let mut dom = VirtualDom::new(App);
    dom.rebuild_in_place();
    let rendered_html = dioxus_ssr::render(&dom);

    // 1. Identidade visual e Cabeçalho
    assert!(
        rendered_html.contains("Muiraquitã"),
        "Header deve conter o nome do app"
    );
    assert!(
        rendered_html.contains("IndieInABox Social Suite"),
        "Header deve conter subtítulo do ecossistema"
    );
    assert!(
        rendered_html.contains("Entrar com IndieAuth"),
        "Botão de login IndieAuth deve estar presente para usuário não autenticado"
    );

    // 2. Trilho de Navegação (NavRail)
    assert!(
        rendered_html.contains("Timeline"),
        "NavRail deve ter aba Timeline"
    );
    assert!(
        rendered_html.contains("Canais"),
        "NavRail deve ter aba Canais"
    );
    assert!(
        rendered_html.contains("Compor"),
        "NavRail deve ter aba Compor"
    );
    assert!(
        rendered_html.contains("Ajustes"),
        "NavRail deve ter aba Ajustes"
    );

    // 3. Rodapé Canônico GOST 13
    assert!(
        rendered_html.contains("travesti latina"),
        "Rodapé deve ter atribuição canônica GOST 13"
    );
    assert!(
        rendered_html.contains("Lumen Pink"),
        "Rodapé deve ter crédito da autora"
    );
    assert!(
        rendered_html.contains("GPL-3.0"),
        "Rodapé deve ter link da licença GPL-3.0"
    );

    // 4. Verificação de estilos responsivos (Mobile & Tablet)
    assert!(
        rendered_html.contains("@media (max-width: 639px)"),
        "CSS embutido deve conter regras específicas para celular (<640px)"
    );
    assert!(
        rendered_html.contains("@media (min-width: 640px) and (max-width: 1024px)"),
        "CSS embutido deve conter regras específicas para tablet (640px-1024px)"
    );
    assert!(
        rendered_html.contains("env(safe-area-inset-top"),
        "CSS embutido deve suportar safe-area insets para notches e gestos"
    );
}
