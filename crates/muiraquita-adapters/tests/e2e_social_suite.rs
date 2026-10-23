//! End-to-End Social Suite Test for Muiraquitã
//! Valida o ciclo completo de uso do protocolo IndieWeb com extensões IIAB:
//! 1. IndieAuth PKCE e descoberta de endpoints via HTML rel
//! 2. Persistência de sessão em storage seguro XDG
//! 3. Consumo de canais e timeline Microsub
//! 4. Voto em enquete descentralizada (_indieinabox_poll)
//! 5. Publicação soberana Micropub com Content Warning e POSSE syndication

use muiraquita_adapters::fakes::FakeSocialStore;
use muiraquita_adapters::http::parse_link_rel;
use muiraquita_adapters::xdg_storage::XdgStorage;
use muiraquita_domain::iiab::{Poll, PollOption};
use muiraquita_domain::indieauth::PkcePair;
use muiraquita_domain::micropub::MicropubDraft;
use muiraquita_domain::microsub::{InteractionType, MicrosubRequest};
use muiraquita_domain::ports::{AuthPort, MicropubPort, MicrosubPort, StoragePort};
use tempfile::tempdir;
use url::Url;

#[tokio::test]
async fn test_full_indieweb_iiab_e2e_lifecycle() {
    println!("=== ETAPA 1: Descoberta de Endpoints & IndieAuth PKCE ===");
    let profile_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <link rel="authorization_endpoint" href="https://auth.lumen.pink/auth">
            <link rel="token_endpoint" href="https://auth.lumen.pink/token">
            <link rel="microsub" href="https://microsub.lumen.pink/api">
            <link rel="micropub" href="https://micropub.lumen.pink/endpoint">
        </head>
        <body><h1 class="h-card">Lumen</h1></body>
        </html>
    "#;

    let base_url = Url::parse("https://lumen.pink/").unwrap();
    let auth_ep = parse_link_rel(profile_html, "authorization_endpoint", &base_url);
    let token_ep = parse_link_rel(profile_html, "token_endpoint", &base_url);
    let microsub_ep = parse_link_rel(profile_html, "microsub", &base_url);
    let micropub_ep = parse_link_rel(profile_html, "micropub", &base_url);

    assert_eq!(
        auth_ep.as_ref().map(|u| u.as_str()),
        Some("https://auth.lumen.pink/auth")
    );
    assert_eq!(
        token_ep.as_ref().map(|u| u.as_str()),
        Some("https://auth.lumen.pink/token")
    );
    assert_eq!(
        microsub_ep.as_ref().map(|u| u.as_str()),
        Some("https://microsub.lumen.pink/api")
    );
    assert_eq!(
        micropub_ep.as_ref().map(|u| u.as_str()),
        Some("https://micropub.lumen.pink/endpoint")
    );

    // Gerar PKCE e autenticar
    let pkce = PkcePair::mock_for_testing();
    let store = FakeSocialStore::seeded();

    let client_id = Url::parse("https://muiraquita.2lp.in/").unwrap();
    let redirect_uri = Url::parse("https://muiraquita.2lp.in/callback").unwrap();
    let token_endpoint = token_ep.as_ref().unwrap();

    let session = store
        .exchange_code(
            token_endpoint,
            "mock-auth-code-12345",
            &base_url,
            &redirect_uri,
            &client_id,
            &pkce,
        )
        .await
        .expect("Troca de código IndieAuth deve suceder");

    assert_eq!(session.me, base_url);
    assert!(session.has_scope("read"));
    assert!(session.has_scope("create"));
    assert!(session.has_scope("channels"));

    println!("=== ETAPA 2: Persistência de Sessão (XDG Base Directory) ===");
    let tmp = tempdir().unwrap();
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");

    let storage = XdgStorage::with_custom_dirs(config_dir, state_dir);
    storage
        .save_session(&session)
        .await
        .expect("Deve salvar sessão");

    let loaded = storage
        .load_session()
        .await
        .expect("Deve carregar sessão")
        .expect("Sessão deve existir");
    assert_eq!(loaded.access_token, session.access_token);
    assert_eq!(loaded.scope, session.scope);
    assert_eq!(loaded.me, session.me);

    println!("=== ETAPA 3: Microsub Channels & Timeline ===");
    let microsub_endpoint = microsub_ep.as_ref().unwrap();
    let channels = store
        .get_channels(microsub_endpoint, &session.access_token)
        .await
        .expect("Deve obter canais Microsub");

    assert!(channels.channels.iter().any(|c| c.uid == "timeline"));
    assert!(channels.channels.iter().any(|c| c.uid == "notifications"));

    let tl_req = MicrosubRequest::Timeline {
        channel: "timeline".to_string(),
        after: None,
        before: None,
        limit: Some(20),
    };
    let timeline = store
        .get_timeline(microsub_endpoint, &session.access_token, &tl_req)
        .await
        .expect("Deve obter timeline inicial");

    assert!(!timeline.items.is_empty(), "Timeline não deve estar vazia");

    // Localizar post com enquete descentralizada
    let poll_entry = timeline
        .items
        .iter()
        .find(|item| {
            item.indieinabox
                .as_ref()
                .map(|ext| ext.poll.is_some())
                .unwrap_or(false)
        })
        .expect("Deve existir um post de teste com enquete");

    let poll_url = poll_entry.url.as_ref().unwrap();
    let initial_votes = poll_entry
        .indieinabox
        .as_ref()
        .unwrap()
        .poll
        .as_ref()
        .unwrap()
        .options[0]
        .votes;

    println!("=== ETAPA 4: Voto em Enquete Descentralizada ===");
    let vote_req = MicrosubRequest::Interact {
        interaction_type: InteractionType::PollVote,
        url: poll_url.clone(),
        content: None,
        poll_option: Some(0),
    };
    store
        .send_action(microsub_endpoint, &session.access_token, &vote_req)
        .await
        .expect("Voto em enquete deve ser registrado com sucesso");

    // Re-consultar timeline e validar incremento de votos
    let timeline_updated = store
        .get_timeline(microsub_endpoint, &session.access_token, &tl_req)
        .await
        .expect("Deve reobter timeline");

    let updated_entry = timeline_updated
        .items
        .iter()
        .find(|e| e.url.as_ref() == Some(poll_url))
        .unwrap();

    let new_votes = updated_entry
        .indieinabox
        .as_ref()
        .unwrap()
        .poll
        .as_ref()
        .unwrap()
        .options[0]
        .votes;

    assert_eq!(
        new_votes,
        initial_votes + 1,
        "Votos da opção da enquete devem ter incrementado em 1"
    );

    println!("=== ETAPA 5: Micropub Composer & Publicação Soberana ===");
    let micropub_endpoint = micropub_ep.as_ref().unwrap();

    let mut draft = MicropubDraft::new_note(
        "Olá do teste E2E do Muiraquitã! Web descentralizada viva e operante.",
    );
    draft.name = Some("Nota E2E Soberana".to_string());
    draft.content_warning = Some("Alerta: Conteúdo de Teste E2E".to_string());
    draft.categories = vec![
        "indieweb".to_string(),
        "rust2024".to_string(),
        "iiab".to_string(),
    ];
    draft.mp_syndicate_to = vec!["https://fediverse.example/@lumen".to_string()];
    draft.poll = Some(Poll::new(
        vec![
            PollOption {
                title: "Sim, 100%".to_string(),
                votes: 1,
            },
            PollOption {
                title: "Com certeza!".to_string(),
                votes: 0,
            },
        ],
        false,
    ));

    let created_url = store
        .create_post(micropub_endpoint, &session.access_token, &draft)
        .await
        .expect("Publicação via Micropub deve suceder")
        .expect("Deve retornar URL do post criado");

    assert!(created_url.starts_with("https://lumen.pink/notes/"));

    // Conferir se o post recém-criado foi injetado na timeline
    let timeline_after_post = store
        .get_timeline(microsub_endpoint, &session.access_token, &tl_req)
        .await
        .expect("Deve obter timeline com novo post");

    let newly_published = timeline_after_post
        .items
        .iter()
        .find(|e| e.url.as_ref() == Some(&created_url))
        .expect("Post publicado deve aparecer na timeline");

    assert_eq!(newly_published.name.as_deref(), Some("Nota E2E Soberana"));
    assert_eq!(
        newly_published.content_warning(),
        Some("Alerta: Conteúdo de Teste E2E")
    );
    assert!(newly_published.indieinabox.as_ref().unwrap().poll.is_some());

    println!("=== SUCESSO: Ciclo E2E completo concluído com 100% de conformidade! ===");
}
