# 🐸 Muiraquitã

[![Build & Test](https://codeberg.org/lumenpink/muiraquita/actions/workflows/ci.yml/badge.svg)](https://codeberg.org/lumenpink/muiraquita)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-red.svg)](https://www.rust-lang.org)
[![Dioxus](https://img.shields.io/badge/Dioxus-0.8.0--alpha.1-orange.svg)](https://dioxuslabs.com)
[![GOST](https://img.shields.io/badge/GOST-v1.1.0-brightgreen.svg)](#padrao-gost)

> **Muiraquitã**: Amuleto verde da Amazônia esculpido em jadeíta ou esteatita, símbolo lendário de cura, coragem e soberania das Icamiabas. Aqui, o amuleto que protege e liberta sua presença digital na web aberta.

**Muiraquitã** é a suíte social soberana integrada ao ecossistema [IndieInABox (IIAB)](https://github.com/lumenpink/indieinabox), implementando 100% dos padrões abertos da **IndieWeb** somados às extensões nativas do IIAB (`_indieinabox`), construída estritamente em **Rust Edição 2024**, **Dioxus 0.8** e seguindo fielmente o padrão **GOST v1.1.0** (perfil `lumen`).

---

## 🌟 Funcionalidades

### 1. Núcleo IndieWeb W3C Compliant
- **IndieAuth (W3C Recommendation / OAuth 2.1)**:
  - Descoberta automática de endpoints (`authorization_endpoint`, `token_endpoint`, `microsub`, `micropub`) via tags `<link rel="...">` HTML5 e cabeçalhos HTTP `Link`.
  - Fluxo seguro com **PKCE** (RFC 7636, `code_challenge_method=S256`).
  - Verificação de tokens imune a timing attacks (`subtle::ConstantTimeEq`).
- **Microsub Reader (IndieWeb Spec)**:
  - Consumo completo de canais e paginação de timelines (`before`, `after`).
  - Normalização e renderização de formatos **JF2** e **Microformats2** (`h-entry`, `h-card`).
  - Interações sociais diretas (likes, reposts, replies e votos).
- **Micropub Composer (W3C Recommendation)**:
  - Criação de notas (`note`), artigos com títulos ricos (`article`), respostas (`reply-to`), favoritos e curtidas.
  - POSSE Syndication (`mp-syndicate-to`).
  - Publicação canônica serializada em JSON `application/json` ou `multipart/form-data`.

### 2. Extensões Nativas IndieInABox (`_indieinabox`)
- **Enquetes Interativas Descentralizadas**:
  - Estrutura de votação soberana (`poll`), suporte a votos anônimos ou assinados por identidade soberana, contagem com barras de progresso percentuais em tempo real.
- **Content Warnings (CW) com Blur Progressivo**:
  - Ocultação inteligente de conteúdo sensível na timeline com revelação sob demanda em clique e aviso personalizável.
  - Compositor integrado com campo dedicado para definir rótulo de alerta sensível.
- **Federated Network Badges**:
  - Destaque visual da origem do post entre ecossistemas abertos: ActivityPub (Mastodon/Fediverse), Twtxt, AT Protocol (Bluesky), Nostr, IndieWeb e RSS/Atom.

---

## 🏗️ Arquitetura Hexagonal (Portas e Adaptadores)

O projeto é organizado como um workspace Cargo modular:

```
muiraquita/
├── crates/
│   ├── muiraquita-domain/      # Núcleo de regras de negócio puras (zero I/O, zero dependência de framework)
│   ├── muiraquita-adapters/    # Implementações de HTTP, XDG Base Directory e Fakes in-memory
│   ├── muiraquita-ui/          # Componentes Dioxus 0.8 (Signals, RSX reativo, tema Baquara)
│   ├── muiraquita-desktop/     # Binário nativo Desktop (Linux, macOS, Windows via WebView)
│   └── muiraquita-web/         # Binário WebAssembly para navegadores
├── site/                       # Vitrine estática hospedada em muiraquita.2lp.in
└── .forgejo/workflows/         # Integração Contínua (CI)
```

### Conformidade XDG Base Directory
- Configurações: `$XDG_CONFIG_HOME/muiraquita/config.json`
- Sessões e credenciais: `$XDG_STATE_HOME/muiraquita/sessions.json`
- Cache temporário: `$XDG_CACHE_HOME/muiraquita/cache/`

---

## 🚀 Como Executar

### Pré-requisitos
- Rust 1.85+ (Edição 2024)
- Dependências de sistema para o WebView nativo (em Linux: `libwebkit2gtk-4.1-dev` ou `libwebkit2gtk-4.0-dev`, `libssl-dev`)

### Rodando a Suíte de Testes
```bash
cargo test --workspace
```

### Rodando o Aplicativo Desktop
```bash
cargo run --bin muiraquita
```

---

## 📜 Padrão GOST

Este repositório segue estritamente a especificação **GOST v1.1.0** (perfil `lumen`):
- `GOSTVER`: 1.1.0
- Versão: SemVer canônico (`VERSION`)
- Cobertura de testes unitários de domínio puro em memória ≥ 90%
- Zero warnings em compilação (`warnings = "deny"`)
- Cabeçalhos de segurança HTTP estritos (`site/_headers`)

---

## 📄 Licença

Distribuído sob a licença **GNU General Public License v3.0 (GPL-3.0)**. Consulte [LICENSE](LICENSE) para mais detalhes.

---

<div align="center">
  <p>Orgulhosamente produzido por uma <a href="https://travesti.2lp.in/pt/" target="_blank" rel="noopener">travesti latina</a> 🏳️‍⚧️ :)</p>
  <p>Muiraquitã v0.1.0 • por <a href="https://lumen.pink" target="_blank" rel="noopener">Lumen Pink</a> • Brasil 🇧🇷 • <a href="https://codeberg.org/lumenpink/muiraquita" target="_blank" rel="noopener">GPL-3.0</a></p>
</div>
