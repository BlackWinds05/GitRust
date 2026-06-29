# GitRust 部署运行指南

从 GitHub 克隆项目到成功运行的全流程指引。

## 环境要求

| 依赖 | 版本要求 | 检查命令 |
|------|----------|----------|
| Rust | ≥ 1.80 | `rustc --version` |
| PostgreSQL | ≥ 14 | `psql --version` |
| Git | ≥ 2.30 | `git --version` |

## 一、安装 Rust（如果还没有）

```bash
# Windows: 下载 rustup-init.exe → https://rustup.rs
# Linux / macOS:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## 二、安装 PostgreSQL

**Windows：** 下载安装包 → https://www.postgresql.org/download/windows/

**macOS：**
```bash
brew install postgresql@16
brew services start postgresql@16
```

**Ubuntu/Debian：**
```bash
sudo apt install postgresql postgresql-client
sudo systemctl start postgresql
```

## 三、安装 sqlx-cli（迁移工具）

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

验证安装：
```bash
sqlx --version
```

## 四、克隆项目

```bash
git clone https://github.com/BlackWinds05/GitRust.git
cd GitRust
```

## 五、创建数据库

```bash
# 根据你的 PostgreSQL 安装方式选择：

# 方式 A：使用 createdb 命令
createdb gitrust

# 方式 B：通过 psql
psql -U postgres -c "CREATE DATABASE gitrust;"

# 方式 C：Windows 下使用 pgAdmin 图形界面创建
```

## 六、配置环境变量

```bash
# 复制配置模板
cp .env.example .env
```

编辑 `.env` 文件，填写必要配置：

```env
# ── 必填 ──
DATABASE_URL=postgres://postgres:你的密码@localhost:5432/gitrust
SESSION_SECRET=随机字符串（至少32字符）
JWT_SECRET=随机字符串（至少32字符）

# ── 可选（有默认值）──
HOST=127.0.0.1
PORT=3000
DATA_DIR=./data
BASE_URL=http://localhost:3000

# ── 邮件（可选，不填则邮件功能不可用）──
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASS=your-app-password
SMTP_FROM=noreply@gitrust.local
```

> **注意：** `DATABASE_URL` 中的 `postgres` 是你的数据库用户名，如果你创建了专用用户则替换之。Windows 下 localhost 可能需要换成 `127.0.0.1`。

### 生成安全的随机密钥（可选）：

```bash
# Linux/macOS
openssl rand -base64 48

# Windows PowerShell
[Convert]::ToBase64String((1..48 | ForEach-Object { Get-Random -Maximum 256 }))
```

## 七、运行数据库迁移

```bash
sqlx migrate run
```

成功输出类似：
```
Applied 202606140001/migrate users (1.234ms)
Applied 202606140002/migrate sessions (0.891ms)
...
Applied 202606140021/migrate transfer_log (0.567ms)
```

## 八、编译运行

```bash
# 编译并运行
cargo run --release
```

首次编译大约需要 3-5 分钟（下载依赖 + 编译）。

成功启动：
```
Listening on http://127.0.0.1:3000
```

## 九、访问

浏览器打开 http://localhost:3000

1. 注册账号（需要验证码）
2. 邮箱验证（配置了 SMTP 才会发送邮件；开发环境可在数据库中手动标记验证）
3. 创建仓库
4. 开始使用

## 常见问题

### 1. `sqlx: command not found`

`sqlx-cli` 未安装或未加入 PATH。确认 `~/.cargo/bin` 在 PATH 中：
```bash
# Linux/macOS
export PATH="$HOME/.cargo/bin:$PATH"

# Windows：将 %USERPROFILE%\.cargo\bin 加入系统 PATH
```

### 2. `connection refused` 或数据库连接失败

检查 PostgreSQL 是否在运行：
```bash
# Linux
sudo systemctl status postgresql

# macOS
brew services list | grep postgresql

# Windows：服务面板中查找 postgresql 服务
```

检查 `.env` 中 `DATABASE_URL` 的用户名、密码、主机、端口是否正确。

### 3. 端口被占用

修改 `.env` 中 `PORT` 为其他值（如 `3001`）。

### 4. 迁移执行失败

检查数据库是否已创建：
```bash
psql -U postgres -l | grep gitrust
```

检查 `DATABASE_URL` 用户是否有建表权限。

### 5. 编译失败

确认 Rust 版本 ≥ 1.80：
```bash
rustc --version
rustup update  # 升级到最新版
```

### 6. Windows 下 Git Bash 路径问题

在 Git Bash 中使用项目路径时，避免中文和空格：
```bash
# 推荐将项目放到英文路径，如：
cd /c/Users/YourName/projects/GitRust
```

### 7. `git2` 编译错误

需要安装 libgit2 开发库：
```bash
# Ubuntu/Debian
sudo apt install libgit2-dev

# macOS
brew install libgit2

# Windows：通常无需额外安装
```

## 数据库表全览（21 个迁移）

| # | 表名 | 说明 |
|---|------|------|
| 001 | users | 用户 |
| 002 | sessions | 会话（tower-sessions） |
| 003 | captcha_challenges | 验证码 |
| 004 | email_verifications | 邮箱验证 |
| 005 | project_groups, group_members | 项目组 |
| 006 | invite_codes | 邀请码 |
| 007 | repositories | 代码仓库 |
| 008 | issues | 缺陷任务 |
| 009 | issue_labels, issue_label_assignments, issue_assignees | 标签系统 |
| 010 | merge_requests | 合并请求 |
| 011 | milestones | 里程碑 |
| 012 | wiki_pages | Wiki 文档 |
| 013 | repository_members | 仓库成员 |
| 014 | ssh_keys | SSH 密钥 |
| 015 | activity_events | 活动动态 |
| 016 | issue_comments | Issue 评论 |
| 017 | issues.due_date 列 | Issue 截止日期 |
| 018 | branch_protection_rules | 分支保护 |
| 020 | repositories 扩展列 | 仓库设置 |
| 021 | repository_transfers | 仓库转让日志 |

## 开发模式

```bash
# 自动重载模板（已内置 minijinja-autoreload）
cargo run

# 监听文件变化自动重启（需要安装 cargo-watch）
cargo install cargo-watch
cargo watch -x run
```

## 重置数据

```bash
# 撤销所有迁移
sqlx migrate revert

# 重新创建
sqlx migrate run

# 或者直接删库重建
dropdb gitrust
createdb gitrust
sqlx migrate run
```
