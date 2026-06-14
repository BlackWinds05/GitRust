# GitRust

**高性能 Rust 在线代码托管平台** — 类 GitHub 的代码协作平台，使用 Rust 构建。

[![Rust](https://img.shields.io/badge/Rust-1.96%2B-orange)](https://www.rust-lang.org)
[![Axum](https://img.shields.io/badge/Axum-0.8-blue)](https://github.com/tokio-rs/axum)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

## 功能特性

### 用户系统
- 注册/登录（含验证码校验）
- 邮箱验证
- 个人资料编辑、密码修改
- SSH 公钥管理

### 项目组
- 创建项目组（团队）
- 邀请码生成（可设有效期/使用次数上限）
- 通过邀请码加入项目组
- 成员角色管理

### 代码仓库
- 创建仓库（个人/项目组）
- 文件树浏览
- README 渲染（Markdown + 语法高亮）
- 提交历史浏览
- 提交详情（含 diff 对比）
- 分支列表
- Git Smart HTTP 协议（支持 `git clone/push`）
- 仓库设置（描述/私密/归档）

### 协作功能
- **Issues** — 缺陷追踪，状态过滤，标签系统
- **Merge Requests** — 分支对比，diff 查看，合并
- **Milestones** — 里程碑管理与进度追踪
- **Wiki** — 页面编辑（Markdown），修订历史
- **Labels** — 标签管理与自定义颜色
- **Members** — 仓库成员权限管理

## 技术栈

| 组件 | 技术 |
|------|------|
| Web 框架 | Axum 0.8 + tower-http |
| 数据库 | PostgreSQL + SQLx 0.8 |
| 模板引擎 | MiniJinja 2.x (SSR) |
| 认证 | argon2 + tower-sessions |
| 验证码 | captcha-rs |
| Git 操作 | git2 (libgit2) |
| Markdown | comrak (GFM) + syntect + ammonia |
| 邮件 | lettre |
| SSH | russh |

## 快速开始

### 前置依赖
- Rust 1.96+
- PostgreSQL 16+
- Git

### 安装运行

```bash
# 克隆仓库
git clone https://github.com/BlackWinds05/GitRust.git
cd GitRust

# 配置环境变量
cp .env.example .env
# 编辑 .env 填写数据库连接等配置

# 创建数据库
createdb gitrust

# 运行迁移
# (Phase 2+ 将在启动时自动创建 sessions 表)

# 编译运行
cargo run --release
```

访问 http://localhost:3000

### 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `DATABASE_URL` | PostgreSQL 连接串 | 必填 |
| `HOST` | 监听地址 | 127.0.0.1 |
| `PORT` | 监听端口 | 3000 |
| `DATA_DIR` | Git 仓库存储目录 | ./data |
| `SESSION_SECRET` | 会话密钥 | 必填 |
| `JWT_SECRET` | JWT 密钥 | 必填 |
| `SMTP_HOST` | 邮件服务器 | (可选) |
| `BASE_URL` | 站点 URL | http://localhost:3000 |

## 项目结构

```
src/
├── main.rs              # 入口 + Axum Router
├── auth/                # 认证（登录/注册/验证码/邮箱）
├── users/               # 用户资料/设置/SSH密钥
├── projects/            # 项目仪表板
├── groups/              # 项目组/邀请码
├── repositories/        # 仓库页面/Git HTTP协议
├── git_core/            # git2 封装（repo/tree/blob/commit/diff）
├── markdown/            # Markdown 渲染（comrak+syntect）
├── issues/              # Issue 追踪
├── labels/              # 标签管理
├── merge_requests/      # 合并请求
├── milestones/          # 里程碑
├── wiki/                # Wiki 页面
├── members/             # 成员管理
├── activity/            # 活动动态
├── ssh/                 # SSH 服务器（规划中）
├── captcha/             # 验证码服务
├── email/               # 邮件服务
├── middleware/           # 认证/日志中间件
└── helpers/             # 分页/slug 工具
```

## 数据库迁移

所有迁移文件位于 `migrations/` 目录：

| # | 表名 |
|---|------|
| 001 | users |
| 002 | sessions |
| 003 | captcha_challenges |
| 004 | email_verifications |
| 005 | project_groups, group_members |
| 006 | invite_codes |
| 007 | repositories |
| 008 | issues |
| 009 | issue_labels, issue_label_assignments, issue_assignees |
| 010 | merge_requests |
| 011 | milestones |
| 012 | wiki_pages |
| 013 | repository_members |
| 014 | ssh_keys |
| 015 | activity_events |

## 开发分支

- `main` — 稳定版本
- `dev_blackwinds` — 开发分支

## License

MIT
