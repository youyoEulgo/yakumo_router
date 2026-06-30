# Yakumo Switch

通用 Anthropic API 兼容反向代理 — 将 Anthropic 模型请求自动路由至任意上游服务。

请求中的模型名根据 **opus / sonnet / haiku** 关键字匹配，替换为 `.env` 中配置的上游模型名。

## 快速开始

### 1. 配置

```bash
cp .env.example .env
# 编辑 .env，填入上游地址、API Key、模型映射
```

`.env` 字段说明：

| 变量 | 含义 |
|------|------|
| `UPSTREAM_BASE_URL` | 上游 Anthropic-compatible API 地址 |
| `API_KEY` | 上游 API Key |
| `MODEL_OPUS` | 请求中包含 "opus" 时替换为的模型名 |
| `MODEL_SONNET` | 请求中包含 "sonnet" 时替换为的模型名 |
| `MODEL_HAIKU` | 请求中包含 "haiku" 时替换为的模型名 |
| `PROXY_PORT` | 监听端口，默认 `8443` |

### 2. 生成 & 信任 TLS 证书

```bash
# step 1: 生成自签名证书 (所有平台通用)
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout key.pem -out cert.pem -days 365 \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"

# step 2: 信任证书 (按平台选择)

# --- macOS ---
sudo security add-trusted-cert -d -r trustRoot \
  -k /Library/Keychains/System.keychain cert.pem

# --- Linux (Debian/Ubuntu 等) ---
sudo cp cert.pem /usr/local/share/ca-certificates/yakumo-switch.crt
sudo update-ca-certificates

# --- Linux (Fedora/CentOS/RHEL 等) ---
sudo cp cert.pem /etc/pki/ca-trust/source/anchors/yakumo-switch.crt
sudo update-ca-trust extract
```

### 3. 运行

```bash
cargo run
```

服务监听 `https://127.0.0.1:8443`。

### 4. 测试

```bash
curl -X POST https://127.0.0.1:8443/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-5",
    "stream": false,
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```