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
[openai]
default_provider = "openrouter"

[openai.providers.openrouter]
base_url = "https://openrouter.ai/api/v1"
api_key = "sk-your-openrouter-key"

[[openai.routes]]
match = "gpt"
provider = "openrouter"
model = "openai/gpt-4.1"

[anthropic]
default_provider = "deepseek"

[anthropic.providers.deepseek]
base_url = "https://api.deepseek.com/anthropic"
api_key = "sk-your-deepseek-key"

[[anthropic.routes]]
match = "sonnet"
provider = "deepseek"
model = "deepseek-v4-pro"
```

Route matching is currently case-insensitive substring matching. If no route
matches, the request is forwarded to the protocol's `default_provider` without
rewriting the model.

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
