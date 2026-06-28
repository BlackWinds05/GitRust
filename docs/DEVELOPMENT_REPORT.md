# GitRust 开发报告

> 基于 Rust 的在线代码托管平台 — 完整开发文档

---

## 6.1 开发环境与技术栈

### 6.1.1 开发环境

| 类别 | 工具 | 版本 |
|------|------|------|
| 操作系统 | Windows 10 Pro | 10.0.19045 |
| 编译器 | Rust (rustc) | 1.96.0 (2026-05-25) |
| 包管理器 | Cargo | 1.96.0 |
| 数据库 | PostgreSQL | 16.x |
| 版本管理 | Git | 2.42.0 |
| 平台仓库 | GitHub | — |

### 6.1.2 后端技术栈

| 类别 | 技术 / Crate | 版本 | 用途 |
|------|-------------|------|------|
| Web 框架 | **Axum** | 0.8 | HTTP 路由、中间件、提取器、WebSocket |
| 异步运行时 | **Tokio** | 1.x | 异步 I/O、多线程调度 |
| HTTP 中间件 | **Tower** / **Tower-HTTP** | 0.5 / 0.6 | CORS、压缩、静态文件、请求追踪 |
| 数据库驱动 | **SQLx** | 0.8 | 异步 PostgreSQL 连接池、类型安全查询 |
| 会话管理 | **Tower-Sessions** | 0.14 | 基于 Cookie 的服务端会话（MemoryStore） |
| 密码哈希 | **Argon2** | 0.5 | 密码单向哈希（m=65536, t=3, p=4） |
| JWT | **jsonwebtoken** | 10 | API Token / WebSocket 认证 |
| 模板引擎 | **MiniJinja** | 2.x | 服务端渲染（SSR），含文件监听热更新 |
| Git 操作 | **git2** (libgit2) | 0.20 | 仓库浏览、提交历史、diff、tree 遍历 |
| 验证码 | **captcha-rs** | 0.5 | 图片验证码生成与校验 |
| Markdown | **comrak** (GFM) | 0.33 | GitHub 风格 Markdown 渲染 |
| 语法高亮 | **syntect** | 5.x | 代码块语法着色 |
| HTML 净化 | **ammonia** | 4.x | XSS 防护、HTML 标签白名单 |
| 邮件 | **lettre** | 0.11 | SMTP 邮件发送（TLS） |
| Base64 | **base64** | 0.22 | HTTP Basic Auth 解码 |
| 其他 | serde / serde_json / uuid / chrono / rand / thiserror / tracing / dotenvy / anyhow | — | 序列化、ID生成、时间处理、随机数、错误处理、日志、环境变量 |

### 6.1.3 前端技术栈

| 类别 | 技术 | 说明 |
|------|------|------|
| 渲染方式 | MiniJinja SSR | 服务端渲染 HTML，无前端框架 |
| 样式 | 原生 CSS | GitHub 风格暗色主题，390 行 |
| 交互 | 原生 JavaScript | 22 行，验证码刷新、Flash 消息 |
| 网络图 | Vis.js (CDN) | Network Graph 可视化 |

### 6.1.4 代码规模统计

| 语言 | 文件数 | 行数 |
|------|--------|------|
| Rust | 85+ | ~4,559 |
| SQL 迁移 | 15 | 205 |
| Jinja 模板 | 40+ | ~1,517 |
| CSS | 1 | 390 |
| JavaScript | 1 | 22 |
| **合计** | **~148** | **~6,693** |

---

## 6.2 项目结构与编码规范

### 6.2.1 目录结构

```
D:\GitRust/
├── Cargo.toml                    # 依赖声明
├── .env.example                  # 环境变量模板
├── .gitignore
├── migrations/                   # 15 个数据库迁移 SQL
├── static/                       # 静态资源
│   ├── css/app.css               # 全局样式（暗色主题）
│   └── js/app.js                 # 前端交互
├── templates/                    # MiniJinja 模板（SSR）
│   ├── base.jinja                # HTML5 骨架
│   ├── layout.jinja              # 页面布局（header/footer）
│   ├── partials/                 # 可复用组件
│   │   ├── header.jinja          # 顶栏导航
│   │   ├── footer.jinja          # 页脚
│   │   ├── sidebar_repo.jinja    # 仓库左侧导航（11 项）
│   │   ├── flash_messages.jinja  # 提示消息
│   │   └── pagination.jinja      # 分页组件
│   ├── pages/
│   │   ├── home.jinja            # 首页/仪表板
│   │   ├── auth/                 # 登录/注册
│   │   ├── user/                 # 个人资料/设置
│   │   ├── projects/             # 项目列表/新建/项目组
│   │   ├── repo/                 # 仓库页面（核心）
│   │   │   ├── overview.jinja    # 文件树 + README
│   │   │   ├── tree.jinja        # 目录浏览（含面包屑）
│   │   │   ├── blob.jinja        # 文件查看（含面包屑）
│   │   │   ├── commits.jinja     # 提交列表
│   │   │   ├── commit.jinja      # 提交详情 + diff
│   │   │   ├── branches.jinja    # 分支列表
│   │   │   ├── graph.jinja       # 网络图
│   │   │   ├── stats.jinja       # 统计图表
│   │   │   ├── settings.jinja    # 仓库设置
│   │   │   ├── labels.jinja      # 标签管理
│   │   │   ├── members.jinja     # 成员管理
│   │   │   ├── issues/           # Issue 列表/详情/新建
│   │   │   ├── merge_requests/   # MR 列表/详情/新建
│   │   │   ├── milestones/       # 里程碑列表
│   │   │   └── wiki/             # Wiki 首页/页面/编辑
│   │   └── errors/               # 403/404/500 错误页
│   └── emails/                   # 邮件模板
└── src/
    ├── main.rs                   # 入口：Axum Router + 中间件栈
    ├── lib.rs                    # 模块导出
    ├── config.rs                 # 环境变量配置
    ├── error.rs                  # 统一错误类型 + HTTP 映射
    ├── state.rs                  # AppState（PgPool + Templates）
    ├── auth/                     # 认证模块
    │   ├── dto.rs                # 表单数据结构
    │   ├── service.rs            # 注册/登录/验证码校验
    │   ├── handlers.rs           # 请求处理器
    │   └── routes.rs             # 路由定义
    ├── captcha/                  # 验证码服务
    ├── email/                    # 邮件服务
    ├── users/                    # 用户资料/设置
    ├── projects/                 # 项目仪表板/创建仓库
    ├── groups/                   # 项目组/邀请码
    ├── repositories/             # 仓库核心
    │   ├── model.rs              # 数据模型
    │   ├── service.rs            # 业务逻辑（创建/解析/权限）
    │   ├── handlers.rs           # 页面处理器（overview/tree/blob/commits/graph/stats/settings）
    │   ├── git_http.rs           # Git Smart HTTP 协议（clone/push）
    │   └── routes.rs             # 路由
    ├── git_core/                 # git2 封装（纯函数，无 DB 依赖）
    │   ├── repo.rs               # 裸仓库初始化/打开
    │   ├── tree.rs               # 文件树遍历 + README 查找
    │   ├── blob.rs               # 文件读取 + 语言检测
    │   ├── commit.rs             # 提交历史/详情/统计
    │   ├── diff.rs               # 差异对比
    │   ├── graph.rs              # 提交图 DAG
    │   └── stats.rs              # 仓库统计
    ├── markdown/                 # Markdown 渲染（comrak+syntect+ammonia）
    ├── issues/                   # Issue 追踪
    ├── labels/                   # 标签管理
    ├── merge_requests/           # 合并请求
    ├── milestones/               # 里程碑
    ├── wiki/                     # Wiki 页面
    ├── members/                  # 仓库成员管理
    ├── activity/                 # 动态推送
    ├── ssh/                      # SSH 服务器（规划中）
    ├── middleware/                # 认证/日志中间件
    └── helpers/                  # 分页/slug 工具
```

### 6.2.2 代码分层逻辑

```
┌──────────────────────────────────────┐
│  routes.rs      路由定义             │  ← axum::Router
├──────────────────────────────────────┤
│  handlers.rs    请求处理器           │  ← 提取参数、调用 service、渲染模板
├──────────────────────────────────────┤
│  service.rs     业务逻辑层           │  ← 数据库操作、业务校验
├──────────────────────────────────────┤
│  model.rs       数据模型             │  ← SQLx FromRow、Serialize
├──────────────────────────────────────┤
│  git_core/      纯函数层（无DB依赖）  │  ← git2 封装，可独立测试
│  markdown/      纯函数层             │  ← comrak 渲染管线
└──────────────────────────────────────┘
```

**依赖规则：**
- `handlers` → `service` → `model` / `git_core` / `markdown` / `PgPool`
- `git_core` 和 `markdown` 不依赖任何其他模块，纯计算
- `config` / `error` / `state` 为基础设施层，被所有模块使用

### 6.2.3 命名规范

| 项目 | 规范 |
|------|------|
| Rust 文件 | snake_case: `git_http.rs`, `merge_requests.rs` |
| Rust 结构体 | PascalCase: `AppState`, `RepoParams`, `GitParams` |
| Rust 函数 | snake_case: `create_repo`, `list_commits` |
| SQL 表名 | snake_case 复数: `users`, `merge_requests` |
| SQL 列名 | snake_case: `owner_type`, `created_at` |
| 模板文件 | snake_case.jinja: `overview.jinja`, `new_repo.jinja` |
| URL 路由 | kebab-case: `/-/merge_requests`, `/-/settings` |
| CSS 类名 | kebab-case: `flash-message`, `btn-primary` |

### 6.2.4 Git 分支策略

| 分支 | 用途 |
|------|------|
| `main` | 稳定版本，合并后触发部署 |
| `dev_blackwinds` | 开发分支，日常推送目标 |

**提交规范：**
- `feat:` — 新功能
- `fix:` — 修复 Bug
- `docs:` — 文档更新
- 每完成一个大功能模块即提交，保持提交历史清晰可追溯

---

## 6.3 核心功能实现说明

### 6.3.1 Git Smart HTTP 协议 — 支持 git clone/push

**功能描述：** 实现 Git 智能 HTTP 协议（Smart HTTP），使用户可通过 `git clone` 和 `git push` 与平台仓库交互。

**实现思路：**

1. **路由设计** — 因 Axum 不支持 `{repo}.git` 单段多参数格式，路由改为 `/{owner}/{repo}/git/info/refs` 等路径
2. **子进程代理** — 使用 `std::process::Command` 调用系统 `git` 二进制处理 pack 协议，避免重新实现复杂的 Git 线协议
3. **认证双通道** — 同时支持 Session Cookie（浏览器）和 HTTP Basic Auth（CLI 推送）

**关键代码片段（info_refs — 引用通告）：**

```rust
pub async fn info_refs(
    State(state): State<Arc<AppState>>,
    session: Session,
    headers: HeaderMap,
    Path(params): Path<GitParams>,
    Query(query): Query<ServiceQuery>,
) -> AppResult<Response> {
    let (repository, _) = service::resolve_repo(&state.pool, &params.owner, &params.repo).await?;
    if repository.is_private && !check_git_auth(&state, &session, &headers).await {
        return Ok(git_unauthorized());  // 返回 401 + WWW-Authenticate 头
    }

    let repo_path = repo::repo_path(&state.config.data_dir, ...);
    let git_subcmd = match query.service.as_str() {
        "git-upload-pack" => "upload-pack",
        "git-receive-pack" => "receive-pack",
        _ => return Err(...)
    };

    // 调用 git 子进程生成引用列表
    let output = std::process::Command::new("git")
        .arg(git_subcmd).arg("--stateless-rpc").arg("--advertise-refs")
        .arg(&repo_path).output()...;

    // Git pkt-line 协议格式：长度前缀 + 服务声明 + 0000 flush + 引用列表
    let pkt = format!("# service={}\n", query.service);
    let pkt_len = format!("{:04x}", pkt.len() + 4);
    let body_data = format!("{}{}0000{}", pkt_len, pkt, ...);

    Ok(Response::builder()
        .header("Content-Type", content_type)
        .body(Body::from(body_data)).unwrap())
}
```

**重难点解决：**

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| clone 失败 `fatal: Could not read` | 缺少 pkt-line flush packet `0000` | 在服务声明后添加 `0000` 分隔符 |
| push 失败 `HTTP 400` | `String` 无法承载二进制 pack 数据 | 改用 `axum::body::Bytes` 直接透传 |
| push 失败 `HTTP 401` | Git 首次请求不发凭据，需 `WWW-Authenticate` 头触发重试 | 返回 `401 + WWW-Authenticate: Basic realm="GitRust"` |
| 日志不显示 | `dotenvy::dotenv()` 在 `tracing_subscriber` 初始化之后加载 | 交换顺序：先加载 `.env`，再初始化日志系统 |
| Fork 创建空仓库 | `git clone --bare` 要求目标目录不存在，但 `init_bare()` 已创建 | clone 前删除空目录，失败时重建 |

### 6.3.2 仓库浏览器 — 文件树 + README + 面包屑导航

**功能描述：** 模拟 GitHub 仓库首页，展示文件列表、渲染 README.md、支持目录递归浏览和面包屑导航。

**实现思路：**

1. **文件树遍历** — 使用 git2 的 `Tree` API 遍历指定路径下的条目，区分为 `Directory`（文件夹）和 `File`
2. **README 发现** — 在根目录查找 `README.md`、`README.markdown`、`README` 等文件名，读取 Blob 内容后通过 comrak 渲染
3. **面包屑导航** — 在 handler 中解析路径为分段数组 `Vec<(name, path)>`，传入模板迭代渲染

**关键代码片段（树遍历）：**

```rust
pub fn list_tree(repo: &Repository, rev: &str, path: &str) -> Result<Vec<FileEntry>, git2::Error> {
    let obj = repo.revparse_single(rev)?;
    let commit = obj.peel_to_commit()?;
    let tree = commit.tree()?;

    let target = if path.is_empty() {
        tree
    } else {
        let entry = tree.get_path(Path::new(path))?;
        repo.find_tree(entry.id())?
    };

    let mut entries = Vec::new();
    for entry in target.iter() {
        let entry_type = match entry.kind() {
            Some(git2::ObjectType::Tree) => "directory",
            Some(git2::ObjectType::Blob) => "file",
            _ => "unknown",
        };
        // 构建完整子路径
        let entry_path = if path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", path.trim_end_matches('/'), name)
        };
        entries.push(FileEntry { name, path: entry_path, entry_type, size });
    }
    // 目录在前，文件在后，各自按字母序
    entries.sort_by(|a, b| /* type priority then alphabetical */);
    Ok(entries)
}
```

**关键代码片段（面包屑生成）：**

```rust
let mut breadcrumbs: Vec<(String, String)> = Vec::new();
if !current_path.is_empty() {
    let mut accum = String::new();
    for part in current_path.split('/') {
        if !part.is_empty() {
            if !accum.is_empty() { accum.push('/'); }
            accum.push_str(part);
            breadcrumbs.push((part.to_string(), accum.clone()));
        }
    }
}
// 传入模板: breadcrumbs → [("src", "src"), ("main", "src/main")]
```

**模板渲染（tree.jinja）：**

```html
{% if breadcrumbs | length > 0 %}
<div style="...">
    <a href="/{{ repo.owner_name }}/{{ repo.name }}/tree?ref_name={{ current_ref }}">
        {{ repo.name }}</a>
    {% for crumb in breadcrumbs %}
        <span> / </span>
        <a href="...?ref_name={{ current_ref }}&path={{ crumb.1 }}">{{ crumb.0 }}</a>
    {% endfor %}
</div>
{% endif %}
```

### 6.3.3 Markdown 渲染管线 — GFM + 语法高亮 + XSS 防护

**功能描述：** 将 Markdown 文本转换为安全的 HTML，支持 GitHub 风格语法、代码块语法高亮。

**实现思路：**

1. **GFM 解析** — 使用 comrak（GitHub 官方 cmark-gfm 的 Rust 移植），启用表格、任务列表、删除线、自动链接、脚注等扩展
2. **语法高亮** — 通过 comrak 的 `SyntectAdapter` 桥接 syntect，使用 `base16-ocean.dark` 主题
3. **安全过滤** — 渲染后的 HTML 经过 ammonia 白名单过滤，移除 `<script>`、`onerror` 等危险标签和属性

```rust
pub fn render_markdown(text: &str) -> String {
    let mut options = ComrakOptions::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.footnotes = true;
    options.extension.header_ids = Some("user-content-".to_string());

    let adapter = SyntectAdapter::new(Some("base16-ocean.dark"));
    let mut plugins = ComrakPlugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let html = markdown_to_html_with_plugins(text, &options, &plugins);
    Builder::default().clean(&html).to_string()  // ammonia 净化
}
```

---

## 6.4 AI 辅助开发说明

### 6.4.1 AI 工具与使用方式

本项目全程使用 **Claude Code**（Anthropic）进行 AI 辅助开发，工具为 CLI 命令行模式。

**操作模式：**
- **Plan Mode** — 需求分析、技术选型研究、架构设计阶段与 AI 交互确认方案
- **Auto Mode** — 代码实现阶段 AI 自动执行编码、编译、调试、提交

### 6.4.2 AI 参与的主要环节

| 环节 | AI 角色 | 具体工作 |
|------|---------|----------|
| **技术选型** | 研究 + 推荐 | 对比 Axum vs Actix-Web、SQLx vs Diesel、comrak vs pulldown-cmark 等，给出推荐方案和理由 |
| **架构设计** | 设计 + 输出 | 设计目录结构、数据库 15 张表的 Schema、80+ URL 路由、模板层级树 |
| **代码实现** | 生成 + 修复 | 生成 Rust handlers/service/model 代码，处理编译错误、依赖冲突、API 版本适配 |
| **Git 工作流** | 提交 + 推送 | 按阶段创建描述性 commit message，推送到 GitHub |
| **调试排错** | 诊断 + 修复 | 分析 captcha 双重前缀、模板找不到、git 协议 401、日志不输出等问题根因 |

### 6.4.3 AI 辅助下的典型开发流程

```
1. [人工] 描述需求："注册/登录需要验证码"
2. [AI] 进入 Plan Mode → 搜索 captcha-rs 用法 → 设计方案
3. [人工] 批准方案
4. [AI] 编写代码 → 编译 → 修复错误循环直到成功
5. [AI] git commit + push
6. [人工] 运行测试 → 发现问题："验证码显示不了"
7. [AI] 读取 captcha-rs 源码 → 发现 to_base64() 已含 data: 前缀 → 修复模板 → push
8. [循环] 直到功能正常
```

### 6.4.4 AI 解决问题的典型案例

**案例 1：Git Smart HTTP 协议调试**

这是整个项目中最复杂的技术问题。AI 经历了 4 轮迭代：
1. 路由 `{repo}.git/...` 被 Axum 拒绝 → 改为 `{repo}/git/...`
2. clone 失败 `fatal: Could not read` → 发现缺少 pkt-line flush packet `0000`
3. push 失败 `HTTP 400` → `String` 无法承载二进制数据 → 改用 `Bytes`
4. push 失败 `HTTP 401` → 发现 Git 认证流程需要 `WWW-Authenticate` 头触发重试

每次迭代 AI 通过读取错误信息 → 分析协议规范 → 修改代码 → 编译验证 → 推送的方式完成。

**案例 2：模板加载失败**

错误信息："template not found: template 'pages/home.jinja' does not exist"

AI 分析思路：
1. 检查模板路径 → 相对路径 `templates/` 是否正确
2. 检查 MiniJinja AutoReloader API → 发现 `watch_path()` 只监听文件变化但不加载模板
3. 检查 MiniJinja 2.x feature → `source` feature 不存在，需要 `loader` feature
4. 最终方案：`env.set_loader(minijinja::path_loader(&dir))`

**案例 3：日志系统不输出**

现象：服务器运行正常但 `git push` 时不显示任何日志

AI 诊断：
1. 怀疑 RUST_LOG 未生效 → 检查 `.env` 文件 → 配置正确
2. 怀疑日志级别过滤 → 改为 `info` 级别 → 仍无输出
3. 最终发现 `dotenvy::dotenv()` 在 `tracing_subscriber::init()` 之后调用 → 交换两行代码顺序 → 解决

### 6.4.5 阶段性开发数据

| 指标 | 数值 |
|------|------|
| 开发阶段 | 10 个 Phase |
| Git 提交数 | 25+ |
| AI 生成代码占比 | ~90% Rust / ~95% 模板 |
| 编译修复迭代 | 平均每 Phase 2-4 轮 |
| 人工介入点 | 需求定义、方案审批、功能测试、PostgreSQL 配置 |

### 6.4.6 AI 辅助开发的体会

**优势：**
- **效率**：10 个完整功能阶段、150+ 文件、~6700 行代码在短时间内完成
- **广度**：AI 能同时处理 Rust 语法、前端模板、数据库 Schema、Git 协议等多个领域
- **调试**：AI 通过编译错误反推根因的能力强，能快速定位第三方 crate 的 API 差异

**经验：**
- 大型任务必须先用 Plan Mode 设计架构，避免盲目编码
- 第三方 crate 的版本 API 差异是主要踩坑点（需 AI 读取源码确认）
- 复杂协议调试（如 Git Smart HTTP）需要多轮迭代，每次只修一个问题
- 人工仍需负责运行测试、配置环境（如 PostgreSQL）、验收功能
