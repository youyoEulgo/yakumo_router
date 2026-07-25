# Yakumo Router

[English README](README.md)

Yakumo Router 是一个用于 OpenAI-compatible / Anthropic-compatible API 的模型路由代理。它会根据请求里的模型名，把请求转发到不同的上游 provider，并提供一个内置 Web UI 来管理 provider、规则和路由表。

项目名取自八云紫的“界线 / 隙间”意象：Yakumo Router 位于客户端和模型供应商之间的边界上，决定一次请求应该穿过哪条“隙间”。

Yakumo Router 不做协议转换：

```text
OpenAI-compatible 请求    -> OpenAI-compatible provider
Anthropic-compatible 请求 -> Anthropic-compatible provider
```

## 功能

- OpenAI-compatible / Anthropic-compatible 请求反向代理
- 按模型名路由，支持 `contains`、`exact`、`regex`
- 可选择是否改写请求里的模型名
- 每种协议可以配置多个 provider
- 路由表支持显式优先级
- 内置 Web UI
- UI 支持中文和英文
- provider、API key、规则和路由表变更可运行时重载
- 检测到本地证书时使用 HTTPS，否则使用 HTTP

## 安装

需要：

- Rust toolchain
- Bun，用于构建前端 UI

从源码构建：

```bash
git clone https://github.com/youyoEulgo/yakumo_router.git
cd yakumo_router

cd ui
bun install
bun run build

cd ..
cargo build --release
```

Cargo 不会自动构建前端；需要先运行 `bun run build` 生成 `ui/dist`，再编译 Rust 二进制。

生成的二进制在：

```text
target/release/yakumo
```

本地开发时也可以直接运行：

```bash
cargo run
```

## 快速开始

1. 克隆项目：

   ```bash
   git clone https://github.com/youyoEulgo/yakumo_router.git
   cd yakumo_router
   ```

2. 构建 Web UI：

   ```bash
   cd ui
   bun install
   bun run build
   cd ..
   ```

3. 启动 Yakumo Router：

   ```bash
   cargo run
   ```

4. 打开 Web UI：

   ```text
   http://127.0.0.1:8989/_ui/
   ```

5. 如果还没有配置文件，点击 **创建配置文件**。

   这会创建一个只包含 server 和 TLS 设置的最小 `config.toml`。配置文件不存在时，除了创建配置文件之外，其他 UI 操作都会锁定。

6. 在 UI 里添加 provider。

   OpenAI-compatible provider 示例：

   ```text
   name: openrouter
   base_url: https://openrouter.ai/api/v1
   api_key: sk-...
   ```

7. 为这个 provider 添加规则。

   示例：

   ```text
   id: openai-gpt
   match: gpt
   match_type: contains
   provider: openrouter
   model: openai/gpt-4.1
   forward_only: false
   ```

8. 把规则加入路由表，并激活路由表。

9. 把你的客户端指向 Yakumo Router：

   ```text
   http://127.0.0.1:8989
   ```

## 配置文件

配置文件会写入用户数据目录：

| 系统 | 目录 |
|----|-----------|
| Linux / BSD | `~/.local/share/yakumo_router/` 或 `$XDG_DATA_HOME/yakumo_router/` |
| macOS | `~/Library/Application Support/yakumo_router/` |
| Windows | `%APPDATA%\yakumo_router\` |

默认配置路径：

```text
<data-dir>/config.toml
```

证书路径默认也相对于这个数据目录解析。

### 最小配置

Web UI 创建的最小配置如下：

```toml
[server]
host = "127.0.0.1"
port = 8989

[tls]
cert = "cert.pem"
key = "key.pem"
```

之后可以继续在 UI 里添加 provider、规则和路由表。

### 完整示例

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

### 字段说明

`server.host`
: 监听 IP。

`server.port`
: 监听端口。

`tls.cert`
: 证书路径。相对路径会从数据目录开始解析。

`tls.key`
: 私钥路径。相对路径会从数据目录开始解析。

`active_route_table`
: 当前启用的路由表名称。如果没有设置，或名称无效，会退回到按规则存储顺序扫描。

`openai.providers.<name>` / `anthropic.providers.<name>`
: 每种协议的 provider 配置。

`base_url`
: 上游 API 地址。

`api_key`
: 发往上游 provider 的 API key，会作为 `Authorization: Bearer ...` 发送。

`[[openai.routes]]` / `[[anthropic.routes]]`
: 每种协议的路由规则。

`id`
: 规则 ID。路由表通过这个 ID 引用规则。

`match`
: 用来匹配请求中 `model` 字段的文本或正则。

`match_type`
: `contains`、`exact` 或 `regex`。匹配不区分大小写。

`provider`
: 命中规则后要使用的 provider 名称。必须属于同一个协议。

`model`
: `forward_only = false` 时，转发给上游的模型名。

`forward_only`
: 为 true 时只选择 provider，不改写模型名。

`route_tables.<name>.openai` / `route_tables.<name>.anthropic`
: 有序规则 ID 列表。越靠前优先级越高。

## 路由逻辑

每次请求进来后，Yakumo Router 会：

1. 判断请求是 OpenAI-compatible 还是 Anthropic-compatible。
2. 读取请求里的 `model`。
3. 查找同协议下的规则。
4. 如果配置了激活路由表，优先按路由表顺序匹配。
5. 选中第一条命中的规则。
6. 如果 `forward_only = false`，改写请求里的 `model`。
7. 转发到规则指定的 provider。

如果没有规则命中，会返回 `400 Bad Request`。

## Web UI

Web UI 地址：

```text
http://127.0.0.1:8989/_ui/
```

可以在 UI 里完成：

- 创建初始最小配置
- 管理 OpenAI-compatible / Anthropic-compatible providers
- 添加、编辑和删除路由规则
- 管理路由表
- 激活路由表
- 拖拽调整路由表内规则优先级
- 切换中文 / 英文

## TLS

当证书和私钥文件都存在时，Yakumo Router 使用 HTTPS；否则使用 HTTP。

在数据目录生成本地证书：

```bash
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout ~/.local/share/yakumo_router/key.pem \
  -out ~/.local/share/yakumo_router/cert.pem \
  -days 365 \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
```

## 开发

前端开发：

```bash
cd ui
bun install
bun run dev
```

构建前端：

```bash
cd ui
bun run build
```

Rust 测试：

```bash
cargo test
```

格式化前端：

```bash
cd ui
bun run format
```

## UI API

普通用户建议直接使用 Web UI。`/_ui/api/*` 是前端内部使用的接口，主要用于调试。

常用调试端点：

```text
GET  /_ui/api/config
POST /_ui/api/config
GET  /_ui/api/providers
GET  /_ui/api/routes
GET  /_ui/api/route-tables
```

当 `config.toml` 不存在时，写操作会返回 `409 Conflict`。

## 许可证

MIT
