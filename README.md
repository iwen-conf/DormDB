# DormDB - 数据库自助申请平台


<img width="2623" height="1312" alt="DormDB" src="https://github.com/user-attachments/assets/82402620-ce77-4e39-8a6d-75f1f8343731" />

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![API Docs](https://img.shields.io/badge/API-Swagger-green.svg)](http://localhost:3000/swagger-ui/)



## 📖 项目简介

DormDB 是一个基于 Rust 开发的数据库自助申请平台，为用户提供安全、自动化的 MySQL 数据库实例申请服务。用户只需提供身份标识，系统即可自动创建独立的数据库、用户账号并分配安全权限。

本项目包含完整的前后端实现，具备用户申请界面、管理员控制面板、完整的 API 接口和安全的权限管理系统。

### 🎯 核心特性

- **🔐 安全第一**: 严格的权限控制，每个用户只能访问自己的数据库
- **⚡ 全自动化**: 一键申请，自动创建数据库、用户和权限配置
- **🛡️ 防重复申请**: 基于身份标识的唯一性校验
- **📊 管理监控**: 完整的申请记录和系统状态监控
- **🎛️ 管理面板**: 现代化的管理员界面，支持用户管理、学号管理和系统监控
- **🔧 事务安全**: 支持失败回滚，确保数据一致性
- **📚 API 文档**: 完整的 Swagger/OpenAPI 文档

## 🏗️ 系统架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Frontend  │    │   Rust Backend  │    │   MySQL Server  │
│                 │    │                 │    │                 │
│ • 申请表单      │◄──►│ • API 服务      │◄──►│ • 动态数据库    │
│ • 结果展示      │    │ • 权限管理      │    │ • 用户管理      │
│ • 管理界面      │    │ • 事务处理      │    │ • 权限控制      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │   SQLite DB     │
                       │                 │
                       │ • 申请记录      │
                       │ • 状态跟踪      │
                       └─────────────────┘
```

## 🚀 快速开始

### 环境要求

- **Rust**: 1.70+
- **MySQL**: 5.7+ 或 8.0+
- **操作系统**: Linux, macOS, Windows

### 安装步骤

1. **克隆项目**
```bash
git clone <repository-url>
cd DormDB
```

2. **配置环境变量**
```bash
cp .env.example .env
# 编辑 .env 文件，设置必要的配置
```

3. **环境变量说明**
```bash
# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# SQLite 配置
SQLITE_PATH=./dormdb_state.db

# MySQL 配置 (必需)
MYSQL_HOST=your-mysql-host
MYSQL_PORT=3306
MYSQL_USERNAME=your-admin-user
MYSQL_PASSWORD=your-admin-password  # 必需
MYSQL_DATABASE=your-database
MYSQL_ALLOWED_HOST=localhost        # 安全限制，不能为 %
```

4. **编译运行**
```bash
# 开发模式
cargo run

# 生产模式
cargo build --release
./target/release/DormDB
```

5. **访问服务**
- **主页**: http://localhost:3000
- **管理界面**: http://localhost:3000/admin.html
- **演示页面**: http://localhost:3000/demo.html
- **API 文档**: http://localhost:3000/swagger-ui/

## 📋 API 接口

### 用户接口

#### 申请数据库
```http
POST /api/v1/apply
Content-Type: application/json

{
  "identity_key": "20250701"
}
```

**响应示例**:
```json
{
  "code": 0,
  "message": "Success",
  "data": {
    "db_host": "localhost",
    "db_port": 3306,
    "db_name": "db_20250701",
    "username": "user_20250701",
    "password": "GeneratedSecurePassword123"
  }
}
```

#### 健康检查
```http
GET /api/v1/health
```

### 管理员接口

#### 获取系统状态
```http
GET /api/v1/admin/status
```

#### 获取申请统计
```http
GET /api/v1/admin/stats
```

#### 数据一致性检查
```http
POST /api/v1/admin/repair
```

## 🔒 安全特性

### 权限控制
- 每个用户只能访问自己的数据库
- 严格限制危险操作权限 (CREATE, DROP, ALTER)
- 禁止访问系统数据库和其他用户数据库

### 授权权限列表
用户获得的权限仅限于：
- `SELECT` - 查询数据
- `INSERT` - 插入数据  
- `UPDATE` - 更新数据
- `DELETE` - 删除数据
- `INDEX` - 创建索引
- `LOCK TABLES` - 锁定表

### 安全限制
- ❌ 禁止 `DROP DATABASE` - 防止删除数据库
- ❌ 禁止 `CREATE DATABASE` - 防止创建新数据库
- ❌ 禁止 `CREATE USER` - 防止创建新用户
- ❌ 禁止访问 `mysql.*` 系统表
- ❌ 禁止跨数据库访问

## 🛠️ 开发指南

### 项目结构
```
src/
├── main.rs           # 应用入口
├── lib.rs            # 库文件
├── api/              # API 路由和处理器
│   └── mod.rs
├── models/           # 数据模型定义
│   └── mod.rs
├── services/         # 业务逻辑层
│   └── mod.rs
├── database/         # 数据库操作层
│   └── mod.rs
├── config/           # 配置管理
│   └── mod.rs
└── utils/            # 工具函数
    └── mod.rs

static/               # 静态文件
├── index.html        # 用户申请页面
├── admin.html        # 管理员界面
├── demo.html         # 演示页面
└── app.js           # 前端逻辑
```

### 运行测试
```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --lib
cargo test --bin
```

## 📊 监控和日志

### 日志级别
- `INFO`: 正常操作记录
- `WARN`: 警告信息
- `ERROR`: 错误信息

### 关键监控指标
- 申请成功率
- 数据库连接状态
- 系统资源使用
- 安全事件记录

## 🔧 故障排查

### 常见问题

1. **用户连接数据库失败** ⭐ 最常见问题
   - **推荐解决方案**: 使用系统提供的**完整连接字符串**
   - 查看管理员界面获取详细连接信息
   - 确保连接工具支持 `allowPublicKeyRetrieval=true` 参数
   - 检查数据库服务器的网络连接性

2. **MySQL 服务器连接失败**
   - 检查 `MYSQL_PASSWORD` 环境变量
   - 验证 MySQL 服务器连接性
   - 确认管理员账号权限

3. **权限不足错误**
   - 确保 MySQL 管理员账号有 `CREATE USER` 权限
   - 检查 `MYSQL_ALLOWED_HOST` 配置

4. **申请失败**
   - 查看应用日志获取详细错误信息
   - 使用管理员接口检查系统状态

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🤝 贡献指南

1. Fork 本项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📚 文档导航

### 核心文档
- **[README.md](README.md)** - 项目概览和快速开始 (本文件)
- **[项目描述.md](项目描述.md)** - 详细项目描述和技术架构
- **[API文档.md](API文档.md)** - 完整的 API 接口文档
- **[部署指南.md](部署指南.md)** - 部署和运维指南

### 技术文档
- **[MYSQL_SECURITY.md](MYSQL_SECURITY.md)** - MySQL 安全配置详解
- **[项目手册.md](项目手册.md)** - 中文项目手册

### 配置文件
- **[.env.example](.env.example)** - 环境变量配置模板
- **[LICENSE](LICENSE)** - MIT 开源许可证

## 📞 支持

如有问题或建议，请：
- 提交 [Issue](../../issues)
- 查看 [在线 API 文档](http://localhost:3000/swagger-ui/)
- 阅读上述相关文档
- 联系开发团队

---

**DormDB Team** - 让数据库申请变得简单安全 🚀
