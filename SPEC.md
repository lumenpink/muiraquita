# Muiraquitã Technical Specification

**Version:** 0.1.0  
**Profile:** lumen / GOST v1.1.0  
**Standard Adherence:** W3C IndieAuth, W3C Micropub, IndieWeb Microsub, JF2, IndieInABox v1.x

---

## 1. Overview & Data Flow

Muiraquitã acts as a sovereign client for the open social web. It can connect directly to any IndieWeb-compliant server (such as WordPress with IndieWeb plugins, Known, Micro.blog, Go-Fed, or custom backends) and seamlessly unlocks extended capabilities when connected to **IndieInABox (IIAB)**.

```
+--------------------+        IndieAuth        +--------------------+
|                    | ----------------------> |                    |
|                    |    (PKCE S256 Flow)     |  IndieAuth Server  |
|                    | <---------------------- |  (Authorization &  |
|                    |      Token + Scopes     |   Token Endpoints) |
|                    |                         +--------------------+
|     Muiraquitã     |
|   (Desktop / Web)  |        Microsub         +--------------------+
|                    | ----------------------> |                    |
|                    |   (Channels & Timelines |  Microsub Server   |
|                    |    + Poll Voting)       |   (IIAB Social     |
|                    | <---------------------- |    Aggregator)     |
|                    |   JF2 Entries + IIAB    +--------------------+
|                    |
|                    |        Micropub         +--------------------+
|                    | ----------------------> |                    |
|                    |   (h-entry JSON + CW    |  Micropub Endpoint |
|                    |    + Polls + Syndication|   (Your Sovereign  |
|                    | <---------------------- |    CMS / Storage)  |
|                    |    201 Created (URL)    +--------------------+
+--------------------+
```

---

## 2. Authentication & Security (IndieAuth)

1. **Endpoint Discovery**:
   - Queries the user's sovereign domain via HTTP GET with custom `User-Agent: Muiraquita/0.1.0`.
   - Parses `<link rel="authorization_endpoint" href="...">` and `<link rel="token_endpoint" href="...">`.
   - Parses `<link rel="microsub" href="...">` and `<link rel="micropub" href="...">`.
2. **PKCE (RFC 7636)**:
   - Random 43-128 character base64url `code_verifier`.
   - `code_challenge = BASE64URL(SHA256(code_verifier))`.
   - Parameter `code_challenge_method = "S256"`.
3. **Constant Time Comparison**:
   - All token comparisons in memory utilize `subtle::ConstantTimeEq` to prevent timing leak side-channels.

---

## 3. IndieInABox Extension Namespace (`_indieinabox`)

Muiraquitã introduces native parsing and authoring for IIAB enhancements inside standard JF2 and Micropub payloads.

### 3.1 Content Warning (`_indieinabox_cw`)
When an entry contains sensitive discussions, spoilers, or triggering themes:
```json
{
  "type": "entry",
  "name": "Spoilers para o final da série",
  "content": {
    "text": "O protagonista descobre que tudo não passava de uma simulação quântica."
  },
  "_indieinabox": {
    "content_warning": "Spoilers da 3ª Temporada",
    "network_origin": "indieweb"
  }
}
```
**UI Behavior:**
The text is blurred (`filter: blur(8px)`) and overlaid with a button indicating `⚠️ Conteúdo Sensível: Spoilers da 3ª Temporada (Clique para revelar)`.

### 3.2 Decentralized Polls (`_indieinabox_poll`)
Micropub posts may include an embedded poll definition:
```json
{
  "type": "entry",
  "content": {
    "text": "Qual o seu modelo favorito de federação?"
  },
  "_indieinabox": {
    "poll": {
      "question": "Qual o seu modelo favorito de federação?",
      "options": [
        { "id": "opt-1", "label": "ActivityPub (Fediverse)", "votes": 42 },
        { "id": "opt-2", "label": "IndieWeb (Webmention + Microsub)", "votes": 78 },
        { "id": "opt-3", "label": "AT Protocol (Bluesky)", "votes": 19 }
      ],
      "closed": false,
      "total_votes": 139
    }
  }
}
```
**Interaction via Microsub:**
Voting is sent via Microsub `action=interact`:
```http
POST /microsub HTTP/1.1
Content-Type: application/x-www-form-urlencoded
Authorization: Bearer <TOKEN>

action=interact&entry=https://lumen.pink/notes/123&interaction=_indieinabox_vote&option_id=opt-2
```

### 3.3 Multi-Network Origins (`network_origin`)
Allowed network tags:
- `indieweb`: Sovereign website / Webmention
- `activitypub`: Fediverse (Mastodon, Misskey, Akkoma)
- `twtxt`: Ultra-minimalist plain text timeline
- `atproto`: Bluesky ecosystem
- `nostr`: Relays descentralizados
- `rss`: Feed clássico RSS/Atom

---

## 4. XDG Base Directory Structure

| Type | Environment Variable | Fallback Path | Purpose |
|------|----------------------|---------------|---------|
| Config | `$XDG_CONFIG_HOME` | `~/.config/muiraquita/config.json` | Server endpoints, theme, active identity |
| State | `$XDG_STATE_HOME` | `~/.local/state/muiraquita/sessions.json` | Access tokens, scopes, auth cookies |
| Cache | `$XDG_CACHE_HOME` | `~/.cache/muiraquita/` | Avatars, media thumbnails, offline feed cache |
