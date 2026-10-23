#[test]
fn test_landing_page_e_readme_amuleto_social() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let site_html =
        std::fs::read_to_string(root.join("site/index.html")).expect("ler site/index.html");
    let readme = std::fs::read_to_string(root.join("README.md")).expect("ler README.md");

    let site_lower = site_html.to_lowercase();
    let readme_lower = readme.to_lowercase();

    let proibidos = ["soberania digital", "leitor microsub, compositor micropub"];

    for termo in proibidos {
        assert!(
            !site_lower.contains(termo),
            "site/index.html ainda contém termo de protocolo/jargão: '{termo}'"
        );
    }

    assert!(
        site_lower.contains("amuleto") && site_lower.contains("sem intermediários"),
        "site/index.html deve focar em amuleto social sem intermediários"
    );
    assert!(
        readme_lower.contains("amuleto"),
        "README.md deve acolher como amuleto"
    );
}
