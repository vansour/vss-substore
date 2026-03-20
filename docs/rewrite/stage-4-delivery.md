# Stage 4 Delivery

## 目标

阶段四把阶段三的“可用原型”推进到“可交付的单运行时版本”：

- 默认运行链路直接服务 Dioxus 构建产物。
- 聚合抓取补齐重定向复检和流式体积限制。
- 管理控制台完成阶段四界面与信息层级收口。

## 本阶段完成

### 运行时与交付链路

- `backend` 暴露为库 crate，二进制共享同一套模块入口。
- Axum 继续从 `WEB_DIST_DIR` 读取 `index.html` 与 `/assets`，默认运行链路保持单服务。
- `Dockerfile` 已改为同时构建：
  - `dx build --platform web --package submora-web --release`
  - `cargo build --release -p submora-server`
- `compose.yml` 已改为直接运行新的阶段四服务和 `dist/` 托管目录。

### 聚合安全加固

- `reqwest` 客户端禁用自动重定向。
- 聚合抓取改为手动跟随重定向，限制最大跳数为 `5`。
- 每次跳转都会重新解析并检查目标地址，继续阻止回环、私网、链路本地和保留地址段。
- 响应体改为流式读取，超出 `10 MiB` 时立即中止，不再只依赖 `Content-Length`。
- HTML 提取仍保留阶段三的正文抽取逻辑，并继续做输出缓冲限制。

### 前端收口

- `AppShell` 升级为阶段四壳层，显式展示统一运行时、技术栈和安全特征。
- 管理控制台新增：
  - 状态提示与错误提示卡片
  - 会话摘要
  - 指标卡片
  - 用户列表卡片化布局
  - 更清晰的源编辑区和账号更新区
- `frontend/assets/app.css` 重写为新的视觉系统，补齐响应式布局、按钮体系、表单和动效。

## 关键文件

- `backend/src/lib.rs`
- `backend/src/main.rs`
- `backend/src/subscriptions.rs`
- `frontend/src/components/shell.rs`
- `frontend/src/components/console.rs`
- `frontend/assets/app.css`
- `Dockerfile`
- `compose.yml`

## 验证结果

本阶段完成后，以下命令已通过：

```bash
cargo check --workspace
```

## 仍保留的约束

- Session 仍使用内存存储，不适合多实例共享登录态。
- Docker 构建依赖 `dioxus-cli 0.7.3`，镜像构建时间会比阶段三更长。
- 仓库中旧实现仍保留，迁移收尾尚未做物理清理。
