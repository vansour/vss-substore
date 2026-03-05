# VSS SubStore — 多用户链接聚合器

VSS SubStore 是一个使用 Rust (Axum) 开发的轻量级、高性能的多用户链接聚合服务。
它可将多个网页的正文抓取并合并为纯文本，以便在客户端或其他服务中消费。

---

## 主要特性 ✅
- 多用户管理：创建、删除、排序用户
- 链接聚合：为每个用户定义若干链接，访问 `/{username}` 返回按顺序合并的纯文本
- 智能 HTML 解析：使用 `scraper` + DOM 遍历保留段落与换行、过滤 `<script>`/`<style>` 等标签
- 管理后台：基于 Cookie 的 Session 认证，前端包含管理面板（`web/`）
- 日志：控制台紧凑输出 + 文件 JSON 格式写入（每天轮转）
- 持久化：SQLite（单文件数据库，零配置）
- 容器友好：Dockerfile + `docker compose` 支持快速部署

---

## 快速开始（推荐：Docker / docker-compose） 🚀
1. 构建并启动容器：

```bash
# 使用本地构建镜像并启动
docker compose up -d --build
```

2. 页面访问：
- 管理后台： http://127.0.0.1:8080
- 订阅合并流（文本）： http://127.0.0.1:8080/{username}

3. 默认管理员：
- 用户名: `admin`
- 密码: `admin`

⚠️ 第一次启动请尽快登录并修改管理员用户名/密码（右上角 账号设置）。

---

## 配置项（环境变量）⚙️

可用环境变量：

- `ADMIN_USER` — 初始化管理员用户名（仅数据库为空时生效）
- `ADMIN_PASSWORD` — 初始化管理员密码（仅数据库为空时生效）
- `DATABASE_URL` — SQLite 数据库路径，示例：`sqlite:///app/data/substore.db`（容器默认）
- `HOST` / `PORT` — 服务器监听地址和端口（默认 `0.0.0.0:8080`）
- `COOKIE_SECURE` — Cookie 是否仅 HTTPS（默认 `false`）
- `LOG_FILE` / `LOG_LEVEL` — 日志文件路径和级别

---

## 构建与本地运行（开发者）🛠️
前提：安装 Rust (stable)

本地运行步骤：

```bash
# 运行服务（数据库会自动创建在 data/substore.db）
cargo run --release
```

---

## Docker 使用建议 🔧
- 容器内的数据目录挂载：把宿主机的 `./data` 挂载到容器 `/app/data` 以持久化数据库。
- 建议使用反向代理（Nginx / Caddy）在生产环境提供 HTTPS；并将 cookie 设置为 secure。

示例 `docker compose`：见 `compose.yml`（默认将 `data` 和 `logs` 映射到宿主机）。

---

## HTTP API（管理/业务接口）📡
所有管理接口需要先通过 Cookie 登录（前端登录会使用 `/api/auth/login`）。

- POST /api/auth/login
  - 请求体: `{ "username": "admin", "password": "admin" }`
  - 登录成功后会设置 Cookie，用于后续管理接口。

- POST /api/auth/logout — 退出登录。
- GET /api/auth/me — 获取当前登录用户。
- PUT /api/auth/account — 更新管理员用户名/密码。

- GET /api/users — 返回按排序的用户名数组（需登录）。
- POST /api/users — 创建用户，body: `{ "username": "foo" }`。
- DELETE /api/users/{username} — 删除用户。
- PUT /api/users/order — 更新用户顺序，body: `{ "order": ["u1","u2"] }`。
- GET /api/users/{username}/links — 获取用户的订阅链接数组。
- PUT /api/users/{username}/links — 更新用户的链接，body: `{ "links": ["https://a.com","https://b.com"] }`。

- GET /{username} — **核心业务接口**，按用户配置的顺序并发抓取每个链接（client 超时 10s，默认并发 10）并返回合并后的纯文本（Content-Type: text/plain）。
- GET /healthz — 健康检查 (HTTP 200 返回 `ok`)。

示例：使用 curl 登录并访问管理接口

```bash
# 登录并保存 cookie 到 cookies.txt
curl -c cookies.txt -X POST -H "Content-Type: application/json" -d '{"username":"admin","password":"admin"}' http://127.0.0.1:8080/api/auth/login

# 使用 cookie 请求用户列表
curl -b cookies.txt http://127.0.0.1:8080/api/users

# 获取某用户合并后的文本
curl http://127.0.0.1:8080/example_user
```

---

## 前端（web UI）📱
前端资源静态保存在 `web/` 目录。登录到管理后台后可以创建用户、配置每个用户的订阅链接、拖拽排序并删除用户。

---

## 日志与监控 📊
- 控制台输出（适合 Docker logs）：紧凑、彩色
- 文件输出（JSON 格式，按天轮转）：路径由环境变量 `LOG_FILE` 指定（默认 `app.log`）。
- 容器镜像拥有健康检查（`HEALTHCHECK` 依赖 `/healthz` 接口）。

---

## 部署和安全建议 🔐
- 在生产环境中：使用 HTTPS（例如 Nginx / Caddy 反向代理）并将 cookie 改为 secure。
- SSRF 风险：应用会对所有配置的链接发起抓取请求，建议在防火墙层限制出站请求或使用网络策略来阻止访问内网地址。
- 超时和并发限制：HTTP 客户端超时时间为 10s，默认并发数为 10。

---

## 贡献与开发建议 🤝
- 请在提交 PR 前确保代码通过 `cargo fmt` 和基本的 `cargo clippy` 检查。
- 数据库表结构在启动时自动创建，无需迁移脚本。

---

## 许可证
本仓库当前没有添加许可证文件。若要发布或允许贡献者复用，请考虑添加合适的开源许可证（例如 MIT / Apache-2.0）。

---

作者: vansour
如果需要 README 中包含更多示例、API 细节或演示截图，请告诉我，我可以进一步补充。
