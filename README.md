# Yakumo Router

[中文 README](README.zh-CN.md)

Yakumo Router is a small reverse proxy for routing OpenAI-compatible and
Anthropic-compatible API requests to different upstream providers by model name.
It includes a Web UI for managing providers, routing rules, and route tables.

The name is inspired by Yakumo Yukari's boundary/gap motif: Yakumo Router sits
on the boundary between clients and model providers, deciding which upstream a
request should cross into.

Yakumo Router does not translate protocols:

```text
OpenAI-compatible request    -> OpenAI-compatible provider
Anthropic-compatible request -> Anthropic-compatible provider
```

## Features

- OpenAI-compatible and Anthropic-compatible reverse proxying
- Model-name routing with `contains`, `exact`, and `regex` match modes
- Optional model rewrite before forwarding
- Multiple providers per protocol
- Route tables with explicit priority order
- Built-in Web UI for configuration
- English and Chinese UI
- Runtime reload for provider, API key, rule, and route-table changes
- HTTPS when local certificate files are present, HTTP otherwise

## Installation

Prerequisites:

- Rust toolchain
- Bun, for building the Web UI

Build from source:

```bash
git clone https://github.com/youyoEulgo/yakumo_router.git
cd yakumo_router

cd ui
bun install
bun run build

cd ..
cargo build --release
```

Cargo does not build the frontend automatically; run `bun run build` first so
`ui/dist` exists before compiling the Rust binary.

The binary is:

```text
target/release/yakumo
```

For local development, you can run it directly with:

```bash
cargo run
```

## Quick Start

1. Clone the project:

   ```bash
   git clone https://github.com/youyoEulgo/yakumo_router.git
   cd yakumo_router
   ```

2. Build the Web UI:

   ```bash
   cd ui
   bun install
   bun run build
   cd ..
   ```

3. Start Yakumo Router:

   ```bash
   cargo run
   ```

4. Open the Web UI:

   ```text
   http://127.0.0.1:8989/_ui/
   ```

5. If no config file exists yet, click **Create config**.

   This creates a minimal `config.toml` with only server and TLS settings.
   Until the config file exists, all other UI actions are locked.

6. Add a provider in the UI.

   Example OpenAI-compatible provider:

   ```text
   name: openrouter
   base_url: https://openrouter.ai/api/v1
   api_key: sk-...
   ```

7. Add a rule for that provider.

   Example:

   ```text
   id: openai-gpt
   match: gpt
   match_type: contains
   provider: openrouter
   model: openai/gpt-4.1
   forward_only: false
   ```

8. Add the rule to a route table and activate that route table.

9. Point your client at Yakumo Router:

   ```text
   http://127.0.0.1:8989
   ```

## Configuration File

The config file is stored in the user data directory:

| OS | Directory |
|----|-----------|
| Linux / BSD | `~/.local/share/yakumo_router/` or `$XDG_DATA_HOME/yakumo_router/` |
| macOS | `~/Library/Application Support/yakumo_router/` |
| Windows | `%APPDATA%\yakumo_router\` |

Default config path:

```text
<data-dir>/config.toml
```

Certificate files are also resolved relative to this directory by default.

### Minimal Config

The Web UI creates this minimal config:

```toml
[server]
host = "127.0.0.1"
port = 8989

[tls]
cert = "cert.pem"
key = "key.pem"
```

You can then add providers, rules, and route tables through the UI.

### Full Example

```toml
active_route_table = "default"

[server]
host = "127.0.0.1"
port = 8989

[tls]
cert = "cert.pem"
key = "key.pem"

[openai.providers.openrouter]
base_url = "https://openrouter.ai/api/v1"
api_key = "sk-your-openrouter-key"

[[openai.routes]]
id = "openai-gpt"
match = "gpt"
match_type = "contains"
provider = "openrouter"
model = "openai/gpt-4.1"
forward_only = false

[anthropic.providers.deepseek]
base_url = "https://api.deepseek.com/anthropic"
api_key = "sk-your-deepseek-key"

[[anthropic.routes]]
id = "anthropic-sonnet"
match = "sonnet"
match_type = "contains"
provider = "deepseek"
model = "deepseek-v4-pro"
forward_only = false

[route_tables.default]
openai = ["openai-gpt"]
anthropic = ["anthropic-sonnet"]
```

### Field Reference

`server.host`
: IP address to listen on.

`server.port`
: Port to listen on.

`tls.cert`
: Certificate path. Relative paths are resolved from the data directory.

`tls.key`
: Private key path. Relative paths are resolved from the data directory.

`active_route_table`
: Name of the route table currently used for matching. If omitted or invalid,
  Yakumo Router falls back to scanning all rules in their stored order.

`openai.providers.<name>` / `anthropic.providers.<name>`
: Provider definitions for each protocol family.

`base_url`
: Upstream API base URL.

`api_key`
: API key sent to the upstream provider as `Authorization: Bearer ...`.

`[[openai.routes]]` / `[[anthropic.routes]]`
: Routing rules for each protocol family.

`id`
: Stable rule ID. Route tables refer to this ID.

`match`
: Text or pattern matched against the request's `model`.

`match_type`
: One of `contains`, `exact`, or `regex`. Matching is case-insensitive.

`provider`
: Provider name to route to. It must exist in the same protocol family.

`model`
: Model name sent upstream when `forward_only = false`.

`forward_only`
: If true, Yakumo Router forwards the request to the selected provider without
  rewriting the model field.

`route_tables.<name>.openai` / `route_tables.<name>.anthropic`
: Ordered rule ID lists. Earlier rules have higher priority.

## Routing Model

For each request, Yakumo Router:

1. Detects whether it is OpenAI-compatible or Anthropic-compatible.
2. Reads the request's `model`.
3. Looks up rules for the same protocol.
4. Checks the active route table order first, if one is configured.
5. Selects the first matching rule.
6. Rewrites `model` unless `forward_only = true`.
7. Forwards the request to the rule's provider.

If no rule matches, the proxy returns `400 Bad Request`.

## Web UI

The Web UI is available at:

```text
http://127.0.0.1:8989/_ui/
```

Use it to:

- Create the initial minimal config
- Manage OpenAI-compatible and Anthropic-compatible providers
- Add, edit, and delete routing rules
- Manage route tables
- Activate a route table
- Drag rules to change priority inside a route table
- Switch between English and Chinese

## TLS

Yakumo Router uses HTTPS when both cert and key files exist. Otherwise it uses
HTTP.

Generate a local certificate in the data directory:

```bash
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout ~/.local/share/yakumo_router/key.pem \
  -out ~/.local/share/yakumo_router/cert.pem \
  -days 365 \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
```

## Development

Frontend:

```bash
cd ui
bun install
bun run dev
```

Production UI build:

```bash
cd ui
bun run build
```

Rust checks:

```bash
cargo test
```

Format frontend:

```bash
cd ui
bun run format
```

## UI API

Most users should use the Web UI. The internal UI API is available under
`/_ui/api/*` and is mainly used by the frontend.

Useful endpoints for debugging:

```text
GET  /_ui/api/config
POST /_ui/api/config
GET  /_ui/api/providers
GET  /_ui/api/routes
GET  /_ui/api/route-tables
```

Mutation endpoints return `409 Conflict` while `config.toml` is missing.

## License

MIT
