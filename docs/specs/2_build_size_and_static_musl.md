# Especificação Técnica: Otimização de Binário e Linking Estático musl

- **Ticket**: #2
- **Escopo**: Configuração do perfil `[profile.release]` e flags de linking estático com musl libc no workspace Rust `muiraquita`.

## 1. Contexto e Motivação
A ausência de configurações explícitas em `[profile.release]` resulta em binários Dioxus/Rust com dezenas de megabytes (contendo símbolos DWARF desnecessários em produção, sem LTO agressivo e com codegen-units paralelas). Além disso, a compilação contra o target `*-unknown-linux-musl` sem `-C target-feature=+crt-static` pode gerar dependência do interpretador dinâmico musl.

## 2. Requisitos Técnicos
1. `Cargo.toml`:
   - `opt-level = "z"` ou `3` (otimização agressiva de tamanho/velocidade).
   - `lto = "fat"`.
   - `codegen-units = 1`.
   - `panic = "abort"`.
   - `strip = true`.
2. `.cargo/config.toml`:
   - Configurar `rustflags = ["-C", "target-feature=+crt-static"]` para os targets `x86_64-unknown-linux-musl` e `aarch64-unknown-linux-musl`.
