# DormDB MySQL 安全权限配置

## 🔒 安全原则

DormDB 采用**最小权限原则**，确保每个用户只能访问和操作自己的数据库，无法影响其他用户或系统数据库。

## 🛡️ 权限配置详情

### 授予的权限 (仅限用户自己的数据库)

```sql
GRANT SELECT, INSERT, UPDATE, DELETE, INDEX, LOCK TABLES ON `db_学号`.* TO 'user_学号'@'localhost'
```

#### ✅ 允许的操作
- **SELECT**: 查询数据
- **INSERT**: 插入新数据
- **UPDATE**: 更新现有数据
- **DELETE**: 删除数据
- **INDEX**: 创建和删除索引
- **LOCK TABLES**: 锁定表进行事务操作

### 明确拒绝的权限

```sql
REVOKE CREATE, DROP, ALTER, REFERENCES, CREATE TEMPORARY TABLES, EXECUTE, 
       CREATE VIEW, SHOW VIEW, CREATE ROUTINE, ALTER ROUTINE, EVENT, TRIGGER 
       ON *.* FROM 'user_学号'@'localhost'
```

#### ❌ 禁止的危险操作
- **CREATE**: 创建新数据库
- **DROP**: 删除数据库或表
- **ALTER**: 修改数据库结构
- **REFERENCES**: 创建外键引用
- **CREATE TEMPORARY TABLES**: 创建临时表
- **EXECUTE**: 执行存储过程
- **CREATE VIEW**: 创建视图
- **SHOW VIEW**: 查看视图定义
- **CREATE ROUTINE**: 创建存储过程/函数
- **ALTER ROUTINE**: 修改存储过程/函数
- **EVENT**: 创建事件调度器
- **TRIGGER**: 创建触发器

## 🔐 安全验证

### 主机限制
- 默认只允许 `localhost` 连接
- 可通过 `MYSQL_ALLOWED_HOST` 环境变量配置特定 IP
- **严禁使用通配符 `%`**

### 用户隔离
每个学号对应：
- 独立的数据库: `db_学号`
- 独立的用户: `user_学号@localhost`
- 仅对自己数据库的有限权限

### 密码安全
- 16位高强度随机密码
- 包含大小写字母、数字、特殊字符
- 每次申请生成新密码

## 🧪 安全测试

### 测试脚本
运行 `./test_database_security.sh` 进行完整的安全测试：

```bash
./test_database_security.sh
```

### 测试项目
1. **基本权限测试**
   - ✅ SELECT 查询
   - ✅ INSERT 插入
   - ✅ UPDATE 更新
   - ✅ DELETE 删除
   - ✅ CREATE TABLE 创建表

2. **安全限制测试**
   - ❌ DROP DATABASE (应被拒绝)
   - ❌ 访问其他数据库 (应被拒绝)
   - ❌ CREATE DATABASE (应被拒绝)
   - ❌ CREATE USER (应被拒绝)

## 🚨 安全风险防护

### 防止的攻击场景

#### 1. 数据库删除攻击
```sql
-- 这些操作会被拒绝
DROP DATABASE db_other_student;
DROP DATABASE mysql;
```

#### 2. 权限提升攻击
```sql
-- 这些操作会被拒绝
CREATE USER 'hacker'@'%' IDENTIFIED BY 'password';
GRANT ALL PRIVILEGES ON *.* TO 'hacker'@'%';
```

#### 3. 跨数据库访问
```sql
-- 这些操作会被拒绝
USE mysql;
SELECT * FROM user;
USE db_other_student;
```

#### 4. 系统表操作
```sql
-- 这些操作会被拒绝
DELETE FROM mysql.user;
UPDATE mysql.user SET Password='hacked';
```

## 🔧 故障排查

### 常见权限错误

#### 1. "Access denied for user"
**原因**: 用户尝试执行被禁止的操作
**解决**: 这是正常的安全限制，不需要修复

#### 2. "Table doesn't exist"
**原因**: 用户尝试访问其他数据库的表
**解决**: 确保只访问自己的数据库

#### 3. "CREATE command denied"
**原因**: 用户尝试创建数据库或用户
**解决**: 这是安全限制，用户只能在自己的数据库内创建表

### 管理员权限检查

#### 检查用户权限
```sql
-- 查看用户权限
SHOW GRANTS FOR 'user_学号'@'localhost';

-- 预期结果应该只包含:
-- GRANT SELECT, INSERT, UPDATE, DELETE, INDEX, LOCK TABLES ON `db_学号`.* TO 'user_学号'@'localhost'
```

#### 检查数据库列表
```sql
-- 查看用户可访问的数据库
SELECT SCHEMA_NAME FROM INFORMATION_SCHEMA.SCHEMATA 
WHERE SCHEMA_NAME LIKE 'db_%';
```

## 📋 权限配置清单

### 部署前检查
- [ ] MySQL 管理员用户有足够权限创建用户和数据库
- [ ] `MYSQL_ALLOWED_HOST` 配置正确 (不使用 `%`)
- [ ] MySQL 服务器禁用了不安全的配置
- [ ] 网络防火墙正确配置

### 运行时监控
- [ ] 定期检查用户权限是否被意外修改
- [ ] 监控异常的数据库访问尝试
- [ ] 检查是否有未授权的数据库或用户创建
- [ ] 验证密码强度和唯一性

## 🎯 最佳实践

### 1. 环境配置
```bash
# 生产环境配置示例
MYSQL_ALLOWED_HOST=10.0.1.100  # 具体IP，不使用通配符
MYSQL_HOST=mysql.internal.com   # 内网MySQL服务器
MYSQL_PORT=3306                 # 标准端口
```

### 2. 网络安全
- 使用内网 MySQL 服务器
- 配置防火墙限制访问
- 启用 MySQL 连接日志
- 定期审计数据库访问

### 3. 监控告警
- 监控失败的权限操作
- 告警异常的数据库创建
- 跟踪用户权限变更
- 记录所有管理员操作

## 🔍 审计日志

### DormDB 日志
```
[申请成功] 身份标识: 20250701, 数据库: db_20250701, 用户: user_20250701
[权限授予] 用户: user_20250701@localhost, 权限: SELECT,INSERT,UPDATE,DELETE,INDEX,LOCK TABLES
[安全限制] 撤销危险权限: CREATE,DROP,ALTER...
```

### MySQL 日志
启用 MySQL 查询日志监控用户操作：
```sql
SET GLOBAL general_log = 'ON';
SET GLOBAL general_log_file = '/var/log/mysql/general.log';
```

## 📞 安全事件响应

### 发现安全问题时
1. **立即隔离**: 禁用相关用户账户
2. **调查范围**: 检查影响的数据库和数据
3. **修复漏洞**: 更新权限配置
4. **通知用户**: 告知受影响的用户
5. **加强监控**: 增加相关的安全检查

### 联系方式
- 安全问题报告: security@example.com
- 紧急响应热线: +86-xxx-xxxx-xxxx
- 技术支持: support@example.com

---

**重要提醒**: 这些安全配置是 DormDB 系统安全的核心，任何修改都应该经过充分的测试和审核。
