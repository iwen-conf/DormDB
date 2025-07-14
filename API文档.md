# DormDB API 文档

## 📖 概述

DormDB 提供 RESTful API 接口，支持数据库自助申请、状态查询和管理功能。所有 API 都返回统一的 JSON 格式响应。

**Base URL**: `http://localhost:3000/api/v1`

**API 文档**: `http://localhost:3000/swagger-ui/`

## 🔧 通用响应格式

所有 API 接口都使用统一的响应格式：

```json
{
  "code": 0,                    // 业务状态码，0表示成功
  "message": "Success",         // 响应消息
  "data": {                     // 响应数据（可选）
    // 具体的响应数据
  }
}
```

### 状态码说明

| 状态码 | 含义 | 说明 |
|--------|------|------|
| 0 | 成功 | 操作成功完成 |
| 40001 | 参数错误 | 请求参数无效或缺失 |
| 40901 | 资源冲突 | 身份标识已存在 |
| 50001 | 内部错误 | 服务器内部错误 |
| 50002 | 数据库操作失败 | 数据库创建或配置失败 |

## 🔐 用户接口

### 1. 申请数据库

创建新的数据库实例和用户账号。

**接口信息**
- **URL**: `/api/v1/apply`
- **方法**: `POST`
- **Content-Type**: `application/json`

**请求参数**
```json
{
  "identity_key": "20250701"    // 用户身份标识，如学号
}
```

**参数说明**
| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| identity_key | string | 是 | 用户唯一身份标识，建议使用学号、工号等 |

**成功响应** (HTTP 200)
```json
{
  "code": 0,
  "message": "Success",
  "data": {
    "db_host": "sql.iluwen.cn",           // 数据库主机地址
    "db_port": 49500,                     // 数据库端口
    "db_name": "db_20250701",             // 数据库名称
    "username": "user_20250701",          // 数据库用户名
    "password": "GeneratedSecurePassword123"  // 数据库密码
  }
}
```

**错误响应示例**

身份标识已存在 (HTTP 409)
```json
{
  "code": 40901,
  "message": "Identity key already exists.",
  "data": null
}
```

参数错误 (HTTP 400)
```json
{
  "code": 40001,
  "message": "Invalid input parameter.",
  "data": null
}
```

**使用示例**
```bash
curl -X POST http://localhost:3000/api/v1/apply \
  -H "Content-Type: application/json" \
  -d '{"identity_key": "20250701"}'
```

### 2. 健康检查

检查服务是否正常运行。

**接口信息**
- **URL**: `/api/v1/health`
- **方法**: `GET`

**成功响应** (HTTP 200)
```json
{
  "code": 0,
  "message": "Success",
  "data": "服务运行正常"
}
```

**使用示例**
```bash
curl http://localhost:3000/api/v1/health
```

## 👨‍💼 管理员接口

### 1. 获取系统状态

获取详细的系统运行状态信息。

**接口信息**
- **URL**: `/api/v1/admin/status`
- **方法**: `GET`

**成功响应** (HTTP 200)
```json
{
  "code": 0,
  "message": "Success",
  "data": {
    "uptime": "2 hours 15 minutes",      // 服务运行时间
    "database_status": "Connected",       // SQLite 连接状态
    "mysql_status": "Connected",          // MySQL 连接状态
    "total_applications": 156,            // 总申请数量
    "today_applications": 23,             // 今日申请数量
    "version": "1.0.0"                    // 系统版本
  }
}
```

**使用示例**
```bash
curl http://localhost:3000/api/v1/admin/status
```

### 2. 获取申请统计

获取申请相关的统计信息。

**接口信息**
- **URL**: `/api/v1/admin/stats`
- **方法**: `GET`

**成功响应** (HTTP 200)
```json
{
  "code": 0,
  "message": "Success",
  "data": {
    "total_applications": 156,            // 总申请数量
    "today_applications": 23,             // 今日申请数量
    "success_rate": 98.5                  // 申请成功率（百分比）
  }
}
```

**使用示例**
```bash
curl http://localhost:3000/api/v1/admin/stats
```

### 3. 获取所有申请者

获取所有已申请用户的详细信息。

**接口信息**
- **URL**: `/api/v1/applicants`
- **方法**: `GET`

**成功响应** (HTTP 200)
```json
{
  "code": 0,
  "message": "Success",
  "data": [
    {
      "id": 1,                            // 申请记录ID
      "identity_key": "20250701",         // 用户身份标识
      "db_name": "db_20250701",           // 数据库名称
      "db_user": "user_20250701",         // 数据库用户名
      "created_at": "2025-07-13T15:00:00Z" // 创建时间
    },
    {
      "id": 2,
      "identity_key": "20250702",
      "db_name": "db_20250702",
      "db_user": "user_20250702",
      "created_at": "2025-07-13T16:30:00Z"
    }
  ]
}
```

**使用示例**
```bash
curl http://localhost:3000/api/v1/applicants
```

### 4. 数据一致性检查和修复

检查 MySQL 和 SQLite 之间的数据一致性，并自动修复不一致的数据。

**接口信息**
- **URL**: `/api/v1/admin/repair`
- **方法**: `POST`

**成功响应** (HTTP 200)
```json
{
  "code": 0,
  "message": "Success",
  "data": "数据一致性检查完成，发现并修复了 2 个不一致项"
}
```

**使用示例**
```bash
curl -X POST http://localhost:3000/api/v1/admin/repair
```

## 🔍 错误处理

### HTTP 状态码

| HTTP 状态码 | 说明 |
|-------------|------|
| 200 | 请求成功 |
| 400 | 请求参数错误 |
| 409 | 资源冲突（如身份标识已存在） |
| 500 | 服务器内部错误 |

### 业务错误码详解

#### 40001 - 参数错误
- **原因**: 请求参数格式错误、缺失必需参数或参数值无效
- **解决**: 检查请求参数格式和内容

#### 40901 - 身份标识已存在
- **原因**: 提供的身份标识已经申请过数据库
- **解决**: 使用不同的身份标识或联系管理员

#### 50001 - 内部错误
- **原因**: 服务器内部处理错误
- **解决**: 查看服务器日志，联系技术支持

#### 50002 - 数据库操作失败
- **原因**: MySQL 数据库创建、用户创建或权限分配失败
- **解决**: 检查 MySQL 服务状态和管理员权限

## 🛠️ 开发工具

### Swagger UI
访问 `http://localhost:3000/swagger-ui/` 可以：
- 查看完整的 API 文档
- 在线测试 API 接口
- 查看请求/响应示例
- 下载 OpenAPI 规范文件

### API 测试
推荐使用以下工具测试 API：
- **curl**: 命令行工具
- **Postman**: 图形化 API 测试工具
- **HTTPie**: 现代化的命令行 HTTP 客户端

### 示例集合

**Postman 集合示例**
```json
{
  "info": {
    "name": "DormDB API",
    "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
  },
  "item": [
    {
      "name": "申请数据库",
      "request": {
        "method": "POST",
        "header": [
          {
            "key": "Content-Type",
            "value": "application/json"
          }
        ],
        "body": {
          "mode": "raw",
          "raw": "{\"identity_key\": \"test_user_001\"}"
        },
        "url": {
          "raw": "{{base_url}}/api/v1/apply",
          "host": ["{{base_url}}"],
          "path": ["api", "v1", "apply"]
        }
      }
    }
  ]
}
```

## 📝 注意事项

1. **身份标识唯一性**: 每个身份标识只能申请一次数据库
2. **密码安全**: 返回的密码是一次性显示，请妥善保存
3. **权限限制**: 创建的数据库用户只能访问自己的数据库
4. **连接限制**: 数据库连接受到主机限制，通常只允许从指定主机连接
5. **管理员接口**: 管理员接口在生产环境中应该添加身份验证

## 🔗 相关链接

- **项目主页**: http://localhost:3000
- **管理界面**: http://localhost:3000/admin.html
- **API 文档**: http://localhost:3000/swagger-ui/
- **GitHub 仓库**: [项目地址]
- **技术文档**: [PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md)
