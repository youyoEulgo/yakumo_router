# Yakumo Switch

OpenAI-compatible / Anthropic-compatible API reverse proxy with configurable
model routing.

Yakumo Switch reads the request protocol, matches the request model name against
the corresponding route table, rewrites the model when a route matches, and
forwards the request to the selected provider.

It does not convert protocols:

```text
OpenAI request    -> OpenAI-compatible provider
Anthropic request -> Anthropic-compatible provider
```

## Config

Create the initial `config.toml` with:

```bash
cargo run -- init
```

The file is written to the user data directory:

| OS | Directory |
|----|-----------|
| Linux / BSD | `~/.local/share/yakumo_switch/` or `$XDG_DATA_HOME/yakumo_switch/` |
| macOS | `~/Library/Application Support/yakumo_switch/` |
| Windows | `%APPDATA%\yakumo_switch\` |

`cert.pem` and `key.pem` are also read from this directory by default.

See `config.example.toml` for the same template.

```toml
active_route_table = "default"

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

Route matching is case-insensitive. `match_type = "contains"` checks whether the
request model contains `match`; `match_type = "exact"` requires the full model
name to match; `match_type = "regex"` treats `match` as a regular expression.
If `forward_only = true`, the request is routed to the selected provider without
rewriting the model. If no route matches, the request returns `400 Bad Request`.
Only one route table is active at a time. Rules in the active table are checked
in order; once one rule matches, later rules are skipped. Route tables store rule
IDs, not full rule definitions.

`config.toml` is watched at runtime. Provider, API key, and route changes are
reloaded automatically after the file is saved. Listener settings such as
`server.host`, `server.port`, and the active TLS/plain HTTP mode still require a
restart.

## Route API

List all providers:

```bash
curl http://127.0.0.1:8989/_ui/api/providers
```

Create or update a provider by name:

```bash
curl -X PUT http://127.0.0.1:8989/_ui/api/providers/openai/openrouter \
  -H "Content-Type: application/json" \
  -d '{
    "base_url": "https://openrouter.ai/api/v1",
    "api_key": "sk-your-openrouter-key"
  }'
```

Delete a provider:

```bash
curl -X DELETE http://127.0.0.1:8989/_ui/api/providers/openai/openrouter
```

Deleting a provider also deletes all rules that reference it.

List all rules:

```bash
curl http://127.0.0.1:8989/_ui/api/routes
```

Create or update a rule by `id`:

```bash
curl -X PUT http://127.0.0.1:8989/_ui/api/routes/openai \
  -H "Content-Type: application/json" \
  -d '{
    "id": "openai-gpt",
    "match": "gpt",
    "match_type": "contains",
    "provider": "openrouter",
    "model": "openai/gpt-4.1",
    "forward_only": false
  }'
```

Use `/_ui/api/routes/anthropic` for Anthropic-compatible rules. If the `id`
already exists in that protocol's route table, it is updated; otherwise a new
rule is appended.

Delete a rule:

```bash
curl -X DELETE http://127.0.0.1:8989/_ui/api/routes/openai/openai-gpt
```

List route tables:

```bash
curl http://127.0.0.1:8989/_ui/api/route-tables
```

Create or update a route table:

```bash
curl -X PUT http://127.0.0.1:8989/_ui/api/route-tables/default \
  -H "Content-Type: application/json" \
  -d '{
    "openai": ["openai-gpt"],
    "anthropic": ["anthropic-sonnet", "anthropic-haiku"]
  }'
```

Activate one route table:

```bash
curl -X PUT http://127.0.0.1:8989/_ui/api/active-route-table/default
```

Delete a route table:

```bash
curl -X DELETE http://127.0.0.1:8989/_ui/api/route-tables/default
```

## TLS

Generate a local certificate into the data directory:

```bash
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout ~/.local/share/yakumo_switch/key.pem \
  -out ~/.local/share/yakumo_switch/cert.pem \
  -days 365 \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
```

When both `cert.pem` and `key.pem` exist, Yakumo Switch listens with HTTPS.
Otherwise it falls back to HTTP.

## Run

```bash
cargo run
```

If `config.toml` does not exist yet, the program exits with a message asking you
to run `yakumo_switch init` first.

Default listener:

```text
https://127.0.0.1:8443
```

or HTTP on the same address when no certificate is found.

## License

MIT
