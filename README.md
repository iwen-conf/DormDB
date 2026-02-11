<div align="center">

# DormDB

**安全的数据库自助申请平台**

<img width="2623" height="1312" alt="DormDB" src="https://github.com/user-attachments/assets/82402620-ce77-4e39-8a6d-75f1f8343731" />

[![Rust](https://img.shields.io/badge/Rust-1.70+-f74c00?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Actix Web](https://img.shields.io/badge/Actix_Web-4-0068d6?style=flat-square)](https://actix.rs)
[![MySQL](https://img.shields.io/badge/MySQL-8.0-4479a1?style=flat-square&logo=mysql&logoColor=white)](https://www.mysql.com)
[![License: MIT](https://img.shields.io/badge/License-MIT-22c55e?style=flat-square)](LICENSE)
[![API Docs](https://img.shields.io/badge/API-Swagger-85ea2d?style=flat-square&logo=swagger&logoColor=black)](http://localhost:3000/swagger-ui/)

[English](README_EN.md) · 中文

</div>

---

## 简介

DormDB 是一个基于 Rust 构建的数据库自助申请平台。用户提供学号，系统自动创建独立的 MySQL 数据库、用户账号并分配安全权限。

包含完整的前后端实现：用户申请界面、管理员控制面板、RESTful API 和 JWT 认证系统。

### 核心特性

| 特性 | 说明 |
|------|------|
| **安全隔离** | 每个用户只能访问自己的数据库，严格禁止跨库操作 |
| **全自动化** | 一键申请，自动创建数据库、用户和权限 |
| **防重复申请** | 基于学号白名单的唯一性校验 |
| **管理面板** | 学号管理、用户管理、系统监控、批量导入 |
| **事务安全** | 失败自动回滚，确保数据一致性 |
| **API 文档** | 内置 Swagger/OpenAPI 文档 |

## 架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Nuxt.js SPA   │◄──►│  Actix-Web API  │◄──►│   MySQL Server  │
│                 │    │                 │    │                 │
│  申请表单       │    │  JWT 认证       │    │  动态数据库     │
│  管理面板       │    │  权限管理       │    │  用户管理       │
│  学号管理       │    │  事务处理       │    │  权限控制       │
└─────────────────┘    └────────┬────────┘    └─────────────────┘
                                │
                       ┌────────▼────────┐
                       │     SQLite      │
                       │   申请记录/状态  │
                       └─────────────────┘
```

## 快速开始

### 环境要求

- Rust 1.70+
- MySQL 5.7+ / 8.0+
- macOS / Linux / Windows

### 安装

```bash
# 克隆
git clone git@github.com:iwen-conf/DormDB.git
cd DormDB

# 配置
cp .env.example .env
# 编辑 .env，填入 MySQL 连接信息

# 运行
cargo run
```

### 环境变量

```bash
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
SQLITE_PATH=./dormdb_state.db

MYSQL_HOST=localhost
MYSQL_PORT=3306
MYSQL_USERNAME=root
MYSQL_PASSWORD=your_password    # 必需
MYSQL_DATABASE=default
MYSQL_ALLOWED_HOST=localhost    # 生产环境禁止使用 %

ADMIN_PASSWORD=admin123         # 管理员密码
DEV_MODE=true                   # 开发模式
RUST_LOG=info
```

### 访问

| 页面 | 地址 |
|------|------|
| 首页 | http://localhost:3000 |
| 管理面板 | http://localhost:3000/admin/dashboard |
| 学号管理 | http://localhost:3000/admin/students |
| API 文档 | http://localhost:3000/swagger-ui/ |

## API

### 用户接口

```http
POST /api/v1/apply
Content-Type: application/json

{ "identity_key": "2023010101" }
```

```json
{
  "code": 0,
  "message": "Success",
  "data": {
    "db_host": "localhost",
    "db_port": 3306,
    "db_name": "db_2023010101",
    "username": "user_2023010101",
    "password": "GeneratedSecurePassword123"
  }
}
```

```http
GET /api/v1/health          # 健康检查
```

### 管理员接口

```http
POST /api/v1/admin/login    # 登录
GET  /api/v1/admin/status   # 系统状态
GET  /api/v1/admin/stats    # 申请统计
POST /api/v1/admin/repair   # 数据一致性修复
GET  /api/v1/admin/students # 学号列表
POST /api/v1/admin/students # 添加学号
```

## 安全

### 用户权限

用户仅获得以下权限，作用域限定在自己的数据库内：

`SELECT` · `INSERT` · `UPDATE` · `DELETE` · `INDEX` · `LOCK TABLES`

### 禁止操作

- `DROP DATABASE` / `CREATE DATABASE` / `CREATE USER`
- 访问 `mysql.*` 系统表
- 跨数据库访问

## 项目结构

```
src/
├── main.rs              # 入口
├── lib.rs               # 库导出
├── api/                 # API 路由和处理器
├── auth/                # JWT 认证和中间件
├── config/              # 配置管理
├── database/            # 数据库操作层
├── models/              # 数据模型
├── routes/              # 路由配置和静态文件服务
├── services/            # 业务逻辑
└── utils/               # 工具函数

static/                  # 前端静态文件 (Nuxt.js SSG)
├── index.html           # 用户申请页
├── admin/               # 管理员页面
│   ├── login/
│   ├── dashboard/
│   └── students/        # 学号管理 (未来)
└── assets/custom.css    # 全局样式覆盖
```

## 开发

```bash
cargo test               # 运行测试
cargo build --release     # 生产构建
RUST_LOG=debug cargo run  # 调试模式
```

## 许可证

[MIT](LICENSE)

## 贡献

1. Fork → 2. 创建分支 → 3. 提交更改 → 4. 发起 PR

---

<div align="center">

**DormDB** — 让数据库申请变得简单安全

</div>
