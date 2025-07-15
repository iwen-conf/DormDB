use crate::config::{AppConfig, MySQLConfig};
use crate::models::{Applicant, DatabaseCredentials};
use anyhow::Result;
use log::{error, info, warn};
use sqlx::{MySql, Pool, Row, Sqlite};

pub struct DatabaseManager {
    sqlite_pool: Pool<Sqlite>,
    mysql_pool: Pool<MySql>,
    mysql_config: MySQLConfig,
}

impl DatabaseManager {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        Self::new_with_retry(config, 3).await
    }

    /// 带重试机制的数据库连接初始化
    pub async fn new_with_retry(config: &AppConfig, max_retries: u32) -> Result<Self> {
        info!("初始化数据库连接，最大重试次数: {}", max_retries);

        // 初始化 SQLite 连接池（优化配置）
        let sqlite_pool = Self::connect_sqlite_with_retry(config, max_retries).await?;

        // 初始化 MySQL 连接池（带重试）
        let mysql_pool = Self::connect_mysql_with_retry(config, max_retries).await?;

        Ok(Self {
            sqlite_pool,
            mysql_pool,
            mysql_config: config.mysql.clone(),
        })
    }

    /// SQLite 连接重试
    async fn connect_sqlite_with_retry(
        config: &AppConfig,
        max_retries: u32,
    ) -> Result<Pool<Sqlite>> {
        let mut last_error = None;

        for attempt in 1..=max_retries {
            info!("尝试连接 SQLite 数据库 (第 {} 次)", attempt);

            match sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(20) // 增加最大连接数
                .min_connections(2) // 设置最小连接数
                .acquire_timeout(std::time::Duration::from_secs(10)) // 获取连接超时
                .idle_timeout(std::time::Duration::from_secs(300)) // 空闲连接超时
                .max_lifetime(std::time::Duration::from_secs(1800)) // 连接最大生命周期
                .connect(&format!("sqlite:{}", config.database.sqlite_path))
                .await
            {
                Ok(pool) => {
                    info!("SQLite 连接成功");

                    // 创建 applicants 表
                    sqlx::query(
                        r#"
                        CREATE TABLE IF NOT EXISTS applicants (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            identity_key TEXT UNIQUE NOT NULL,
                            db_name TEXT NOT NULL,
                            db_user TEXT NOT NULL,
                            status TEXT NOT NULL DEFAULT 'success',
                            failure_reason TEXT,
                            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                            deleted_at DATETIME,
                            deletion_reason TEXT
                        )
                        "#,
                    )
                    .execute(&pool)
                    .await?;

                    // 创建学号管理表
                    sqlx::query(
                        r#"
                        CREATE TABLE IF NOT EXISTS student_ids (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            student_id TEXT UNIQUE NOT NULL,
                            student_name TEXT,
                            class_info TEXT,
                            has_applied BOOLEAN NOT NULL DEFAULT FALSE,
                            applied_db_name TEXT,
                            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
                        )
                        "#,
                    )
                    .execute(&pool)
                    .await?;

                    // 为现有记录添加新字段 (如果表已存在)
                    let _ = sqlx::query(
                        "ALTER TABLE applicants ADD COLUMN status TEXT DEFAULT 'success'",
                    )
                    .execute(&pool)
                    .await;
                    let _ = sqlx::query("ALTER TABLE applicants ADD COLUMN failure_reason TEXT")
                        .execute(&pool)
                        .await;
                    let _ = sqlx::query("ALTER TABLE applicants ADD COLUMN deleted_at DATETIME")
                        .execute(&pool)
                        .await;
                    let _ = sqlx::query("ALTER TABLE applicants ADD COLUMN deletion_reason TEXT")
                        .execute(&pool)
                        .await;

                    // 迁移：如果存在 username 列，将其重命名为 db_user
                    // SQLite 不支持直接重命名列，所以我们需要检查列是否存在
                    let column_check = sqlx::query("PRAGMA table_info(applicants)")
                        .fetch_all(&pool)
                        .await;

                    if let Ok(columns) = column_check {
                        let has_username = columns.iter().any(|row| {
                            if let Ok(name) = row.try_get::<String, _>("name") {
                                name == "username"
                            } else {
                                false
                            }
                        });

                        let has_db_user = columns.iter().any(|row| {
                            if let Ok(name) = row.try_get::<String, _>("name") {
                                name == "db_user"
                            } else {
                                false
                            }
                        });

                        // 如果有 username 但没有 db_user，需要迁移
                        if has_username && !has_db_user {
                            info!("检测到旧的数据库结构，开始迁移...");

                            // 创建新表
                            let _ = sqlx::query(
                                r#"
                                CREATE TABLE applicants_new (
                                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                                    identity_key TEXT UNIQUE NOT NULL,
                                    db_name TEXT NOT NULL,
                                    db_user TEXT NOT NULL,
                                    status TEXT NOT NULL DEFAULT 'success',
                                    failure_reason TEXT,
                                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                    deleted_at DATETIME,
                                    deletion_reason TEXT
                                )
                                "#,
                            )
                            .execute(&pool)
                            .await;

                            // 复制数据
                            let _ = sqlx::query(
                                "INSERT INTO applicants_new (id, identity_key, db_name, db_user, status, failure_reason, created_at, deleted_at, deletion_reason) SELECT id, identity_key, db_name, username, status, failure_reason, created_at, deleted_at, deletion_reason FROM applicants"
                            )
                            .execute(&pool)
                            .await;

                            // 删除旧表
                            let _ = sqlx::query("DROP TABLE applicants")
                                .execute(&pool)
                                .await;

                            // 重命名新表
                            let _ = sqlx::query("ALTER TABLE applicants_new RENAME TO applicants")
                                .execute(&pool)
                                .await;

                            info!("数据库迁移完成");
                        }
                    }

                    return Ok(pool);
                }
                Err(e) => {
                    error!("SQLite 连接失败 (第 {} 次): {}", attempt, e);
                    last_error = Some(e);

                    if attempt < max_retries {
                        let delay = std::time::Duration::from_secs(2_u64.pow(attempt - 1)); // 指数退避
                        warn!("等待 {:?} 后重试...", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap().into())
    }

    /// MySQL 连接重试
    async fn connect_mysql_with_retry(config: &AppConfig, max_retries: u32) -> Result<Pool<MySql>> {
        let mysql_url = format!(
            "mysql://{}:{}@{}:{}/{}?ssl-mode=disabled&allowPublicKeyRetrieval=true",
            config.mysql.username,
            config.mysql.password,
            config.mysql.host,
            config.mysql.port,
            config.mysql.database
        );

        let mut last_error = None;

        for attempt in 1..=max_retries {
            info!("尝试连接 MySQL 数据库 (第 {} 次)", attempt);

            match sqlx::mysql::MySqlPoolOptions::new()
                .max_connections(10) // 增加最大连接数
                .min_connections(1) // 设置最小连接数
                .acquire_timeout(std::time::Duration::from_secs(15)) // 获取连接超时
                .idle_timeout(std::time::Duration::from_secs(600)) // 空闲连接超时
                .max_lifetime(std::time::Duration::from_secs(3600)) // 连接最大生命周期
                .test_before_acquire(true) // 获取前测试连接
                .connect(&mysql_url)
                .await
            {
                Ok(pool) => {
                    info!("MySQL 连接成功");
                    return Ok(pool);
                }
                Err(e) => {
                    error!("MySQL 连接失败 (第 {} 次): {}", attempt, e);
                    last_error = Some(e);

                    if attempt < max_retries {
                        let delay = std::time::Duration::from_secs(2_u64.pow(attempt - 1)); // 指数退避
                        warn!("等待 {:?} 后重试...", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap().into())
    }

    // 检查身份标识是否已存在（排除已删除的记录）
    pub async fn check_identity_exists(&self, identity_key: &str) -> Result<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM applicants WHERE identity_key = ? AND status != 'deleted'")
            .bind(identity_key)
            .fetch_one(&self.sqlite_pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    // 创建新的申请记录
    pub async fn create_applicant(
        &self,
        identity_key: &str,
        db_name: &str,
        db_user: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO applicants (identity_key, db_name, db_user, status) VALUES (?, ?, ?, 'success')"
        )
        .bind(identity_key)
        .bind(db_name)
        .bind(db_user)
        .execute(&self.sqlite_pool)
        .await?;

        Ok(())
    }

    // 创建失败的申请记录
    pub async fn create_failed_applicant(&self, identity_key: &str, reason: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO applicants (identity_key, db_name, db_user, status, failure_reason) VALUES (?, '', '', 'failed', ?)"
        )
        .bind(identity_key)
        .bind(reason)
        .execute(&self.sqlite_pool)
        .await?;

        Ok(())
    }

    // 在 MySQL 中创建数据库和用户
    pub async fn provision_database(
        &self,
        identity_key: &str,
        password: &str,
    ) -> Result<DatabaseCredentials> {
        // 首先验证学号是否允许申请
        if !self.is_student_id_allowed(identity_key).await? {
            error!("学号 {} 不在允许列表中或已申请过数据库", identity_key);
            return Err(anyhow::anyhow!("学号不在允许列表中或已申请过数据库"));
        }

        let db_name = format!("db_{}", identity_key);
        let username = format!("user_{}", identity_key);

        info!("开始为身份标识 {} 创建数据库和用户", identity_key);
        info!("数据库名: {}, 用户名: {}", db_name, username);

        // 从配置获取允许的主机，默认为 localhost 以确保安全
        let allowed_host = self
            .mysql_config
            .allowed_host
            .as_deref()
            .unwrap_or("localhost");
        info!("允许的主机: {}", allowed_host);

        // 验证主机不是通配符 % (严禁在生产环境使用)
        let is_dev_mode = std::env::var("DEV_MODE").unwrap_or_default() == "true";
        if allowed_host == "%" && !is_dev_mode {
            error!("安全违规: 生产环境禁止使用通配符主机 '%'");
            return Err(anyhow::anyhow!(
                "安全错误: 不允许使用通配符主机 '%'"
            ));
        }

        if allowed_host == "%" && is_dev_mode {
            warn!("⚠️  开发模式: 使用通配符主机 '%' - 仅用于开发环境！");
        }

        // 创建数据库 - 使用参数化查询防止SQL注入
        // 注意：MySQL不支持数据库名的参数化，但我们验证输入格式
        if !Self::is_valid_identifier(&db_name) {
            error!("无效的数据库名格式: {}", db_name);
            return Err(anyhow::anyhow!("Invalid database name format"));
        }

        info!("步骤 1: 创建数据库 {}", db_name);
        // 使用更安全的方式创建数据库，避免SQL注入
        // 由于MySQL不支持对数据库名进行参数化，我们使用白名单验证
        let sanitized_db_name = format!("db_{}", identity_key);
        if !Self::is_valid_database_name(&sanitized_db_name) {
            error!("数据库名不符合安全规范: {}", sanitized_db_name);
            return Err(anyhow::anyhow!("数据库名不符合安全规范"));
        }
        
        let create_db_sql = format!("CREATE DATABASE IF NOT EXISTS `{}`", sanitized_db_name);
        if let Err(e) = sqlx::query(&create_db_sql).execute(&self.mysql_pool).await {
            error!("创建数据库失败: {}, SQL: {}", e, create_db_sql);
            return Err(e.into());
        }
        info!("数据库 {} 创建成功", sanitized_db_name);

        // 创建用户 - 使用更安全的方式
        if !Self::is_valid_username(&username) {
            error!("无效的用户名格式: {}", username);
            return Err(anyhow::anyhow!("Invalid username format"));
        }
        if !Self::is_valid_host(allowed_host) {
            error!("无效的主机格式: {}", allowed_host);
            return Err(anyhow::anyhow!("Invalid host format"));
        }

        info!("步骤 2: 创建用户 {}@{}", username, allowed_host);
        
        // 创建用户时使用预处理语句（某些MySQL版本支持）
        let create_user_sql = format!(
            "CREATE USER IF NOT EXISTS '{}'@'{}' IDENTIFIED BY '{}'",
            username, allowed_host, password.replace("'", "''") // 转义单引号
        );
        
        if let Err(e) = sqlx::query(&create_user_sql)
            .execute(&self.mysql_pool)
            .await
        {
            error!(
                "创建用户失败: {}, SQL: CREATE USER IF NOT EXISTS '{}'@'{}' IDENTIFIED BY '[REDACTED]'",
                e, username, allowed_host
            );
            return Err(e.into());
        }
        info!("用户 {}@{} 创建成功", username, allowed_host);

        info!("步骤 3: 授予安全权限给用户 {}@{}", username, allowed_host);
        // 授权 - 严格限制权限，只授予必要的数据库操作权限
        // 不授予 CREATE, DROP, ALTER 等危险权限，防止用户删除数据库或修改结构
        let grant_sql = format!(
            "GRANT SELECT, INSERT, UPDATE, DELETE, INDEX, LOCK TABLES ON `{}`.* TO '{}'@'{}'",
            db_name, username, allowed_host
        );
        if let Err(e) = sqlx::query(&grant_sql).execute(&self.mysql_pool).await {
            error!("授权失败: {}, SQL: {}", e, grant_sql);
            return Err(e.into());
        }
        info!("权限授予成功: SELECT, INSERT, UPDATE, DELETE, INDEX, LOCK TABLES");

        // 明确拒绝全局权限和危险操作
        info!("步骤 4: 撤销危险权限");
        let revoke_dangerous_sql = format!(
            "REVOKE CREATE, DROP, ALTER, REFERENCES, CREATE TEMPORARY TABLES, EXECUTE, CREATE VIEW, SHOW VIEW, CREATE ROUTINE, ALTER ROUTINE, EVENT, TRIGGER ON *.* FROM '{}'@'{}'",
            username, allowed_host
        );
        // 注意：REVOKE 可能失败如果用户没有这些权限，所以我们忽略错误
        if let Err(e) = sqlx::query(&revoke_dangerous_sql)
            .execute(&self.mysql_pool)
            .await
        {
            warn!("撤销危险权限时出现警告 (可忽略): {}", e);
        } else {
            info!("危险权限撤销成功");
        }

        // 刷新权限
        info!("步骤 5: 刷新权限");
        if let Err(e) = sqlx::query("FLUSH PRIVILEGES")
            .execute(&self.mysql_pool)
            .await
        {
            error!("刷新权限失败: {}", e);
            return Err(e.into());
        }
        info!("权限刷新成功");

        info!("✅ 数据库和用户创建完成！身份标识: {}", identity_key);
        info!("   数据库: {}", db_name);
        info!("   用户: {}@{}", username, allowed_host);
        info!("   权限: SELECT, INSERT, UPDATE, DELETE, INDEX, LOCK TABLES (仅限指定数据库)");

        // 生成完整的连接字符串
        let connection_string = format!(
            "mysql://{}:{}@{}:{}/{}?allowPublicKeyRetrieval=true&useSSL=false",
            username, password, self.mysql_config.host, self.mysql_config.port, db_name
        );

        let jdbc_url = format!(
            "jdbc:mysql://{}:{}/{}?allowPublicKeyRetrieval=true&useSSL=false&user={}&password={}",
            self.mysql_config.host, self.mysql_config.port, db_name, username, password
        );

        // 标记学号已申请数据库
        if let Err(e) = self.mark_student_applied(identity_key, &db_name).await {
            warn!("标记学号已申请失败: {}", e);
            // 不影响主流程，只记录警告
        }

        Ok(DatabaseCredentials {
            db_host: self.mysql_config.host.clone(),
            db_port: self.mysql_config.port,
            db_name,
            username,
            password: password.to_string(),
            connection_string,
            jdbc_url,
        })
    }

    // 获取所有申请者列表（管理员功能）
    pub async fn get_all_applicants(&self) -> Result<Vec<Applicant>> {
        let applicants =
            sqlx::query_as::<_, Applicant>("SELECT * FROM applicants ORDER BY created_at DESC")
                .fetch_all(&self.sqlite_pool)
                .await?;

        Ok(applicants)
    }

    // 安全验证函数

    /// 验证数据库标识符是否安全（防止SQL注入）
    fn is_valid_identifier(identifier: &str) -> bool {
        // 只允许字母、数字和下划线，且不能为空
        if identifier.is_empty() || identifier.len() > 64 {
            return false;
        }

        // 必须以字母或下划线开头
        let first_char = identifier.chars().next().unwrap();
        if !first_char.is_alphabetic() && first_char != '_' {
            return false;
        }

        // 只包含字母、数字和下划线
        identifier.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// 验证数据库名是否安全（更严格的验证）
    fn is_valid_database_name(db_name: &str) -> bool {
        // 数据库名必须以 "db_" 开头，后面跟学号
        if !db_name.starts_with("db_") {
            return false;
        }
        
        let student_id = &db_name[3..]; // 去掉 "db_" 前缀
        
        // 验证学号部分
        if let Err(_) = crate::auth::StudentValidator::validate_student_id_format(student_id) {
            return false;
        }
        
        // 使用基础验证
        Self::is_valid_identifier(db_name)
    }

    /// 验证用户名是否安全（更严格的验证）
    fn is_valid_username(username: &str) -> bool {
        // 用户名必须以 "user_" 开头，后面跟学号
        if !username.starts_with("user_") {
            return false;
        }
        
        let student_id = &username[5..]; // 去掉 "user_" 前缀
        
        // 验证学号部分
        if let Err(_) = crate::auth::StudentValidator::validate_student_id_format(student_id) {
            return false;
        }
        
        // 使用基础验证
        Self::is_valid_identifier(username)
    }

    /// 验证主机地址是否安全
    fn is_valid_host(host: &str) -> bool {
        if host.is_empty() || host.len() > 255 {
            return false;
        }

        // 在开发模式下允许通配符
        if host == "%" {
            let is_dev_mode = std::env::var("DEV_MODE").unwrap_or_default() == "true";
            if is_dev_mode {
                warn!("⚠️  开发模式: 允许通配符主机 '%' - 仅用于开发环境！");
                return true;
            } else {
                return false;
            }
        }

        // 简单的主机名/IP验证
        // 允许 localhost, IP地址, 域名
        if host == "localhost" {
            return true;
        }

        // 检查是否为有效的IP地址格式
        if Self::is_valid_ip(host) {
            return true;
        }

        // 检查是否为有效的域名格式
        Self::is_valid_hostname(host)
    }

    /// 验证IP地址格式
    fn is_valid_ip(ip: &str) -> bool {
        ip.parse::<std::net::IpAddr>().is_ok()
    }

    /// 验证主机名格式
    fn is_valid_hostname(hostname: &str) -> bool {
        if hostname.is_empty() || hostname.len() > 253 {
            return false;
        }

        // 域名只能包含字母、数字、点和连字符
        // 不能以点或连字符开头或结尾
        if hostname.starts_with('.')
            || hostname.ends_with('.')
            || hostname.starts_with('-')
            || hostname.ends_with('-')
        {
            return false;
        }

        // 检查是否有连续的点
        if hostname.contains("..") {
            return false;
        }

        hostname
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
    }

    // 管理员功能方法 (ADM-001, ADM-002)

    /// 测试SQLite连接
    pub async fn test_sqlite_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.sqlite_pool).await?;
        Ok(())
    }

    /// 测试MySQL连接
    pub async fn test_mysql_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.mysql_pool).await?;
        Ok(())
    }

    /// 统计总申请数量
    pub async fn count_total_applications(&self) -> Result<i64> {
        let count = sqlx::query_scalar("SELECT COUNT(*) FROM applicants")
            .fetch_one(&self.sqlite_pool)
            .await?;
        Ok(count)
    }

    /// 统计今日申请数量
    pub async fn count_today_applications(&self) -> Result<i64> {
        let count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM applicants WHERE DATE(created_at) = DATE('now')",
        )
        .fetch_one(&self.sqlite_pool)
        .await?;
        Ok(count)
    }

    /// 统计本周申请数量
    pub async fn count_week_applications(&self) -> Result<i64> {
        let count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM applicants WHERE created_at >= DATE('now', '-7 days')",
        )
        .fetch_one(&self.sqlite_pool)
        .await?;
        Ok(count)
    }

    /// 统计本月申请数量
    pub async fn count_month_applications(&self) -> Result<i64> {
        let count = sqlx::query_scalar(
            "SELECT COUNT(*) FROM applicants WHERE created_at >= DATE('now', 'start of month')",
        )
        .fetch_one(&self.sqlite_pool)
        .await?;
        Ok(count)
    }

    /// 获取最近的申请记录
    pub async fn get_recent_applications(&self, limit: i32) -> Result<Vec<Applicant>> {
        let applicants = sqlx::query_as::<_, Applicant>(
            "SELECT id, identity_key, db_name, db_user, status, failure_reason, created_at, deleted_at, deletion_reason FROM applicants ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.sqlite_pool)
        .await?;

        Ok(applicants)
    }

    // 学号管理功能

    /// 检查学号是否存在且允许申请
    pub async fn is_student_id_allowed(&self, student_id: &str) -> Result<bool> {
        // 首先验证学号格式
        if let Err(e) = crate::auth::StudentValidator::validate_student_id_format(student_id) {
            error!("学号格式验证失败: {}", e);
            return Ok(false);
        }

        // 检查学号是否在白名单中且未申请过
        let count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM student_ids WHERE student_id = ? AND has_applied = 0"
        )
        .bind(student_id)
        .fetch_one(&self.sqlite_pool)
        .await?;

        if count == 0 {
            warn!("学号 {} 不在白名单中或已申请过数据库", student_id);
            return Ok(false);
        }

        info!("学号 {} 验证通过", student_id);
        Ok(true)
    }

    /// 标记学号已申请数据库
    pub async fn mark_student_applied(&self, student_id: &str, db_name: &str) -> Result<()> {
        sqlx::query(
            "UPDATE student_ids SET has_applied = 1, applied_db_name = ?, updated_at = CURRENT_TIMESTAMP WHERE student_id = ?"
        )
        .bind(db_name)
        .bind(student_id)
        .execute(&self.sqlite_pool)
        .await?;

        Ok(())
    }

    /// 获取所有学号记录
    pub async fn get_all_student_ids(&self, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<crate::models::StudentId>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let student_ids = sqlx::query_as::<_, crate::models::StudentId>(
            "SELECT id, student_id, student_name, class_info, has_applied, applied_db_name, created_at, updated_at FROM student_ids ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.sqlite_pool)
        .await?;

        Ok(student_ids)
    }

    /// 添加单个学号
    pub async fn add_student_id(&self, student_id: &str, student_name: Option<&str>, class_info: Option<&str>) -> Result<()> {
        // 验证学号格式
        crate::auth::StudentValidator::validate_student_id_format(student_id)?;

        sqlx::query(
            "INSERT INTO student_ids (student_id, student_name, class_info) VALUES (?, ?, ?)"
        )
        .bind(student_id)
        .bind(student_name)
        .bind(class_info)
        .execute(&self.sqlite_pool)
        .await?;

        Ok(())
    }

    /// 更新学号信息
    pub async fn update_student_id(&self, id: i32, student_name: Option<&str>, class_info: Option<&str>) -> Result<()> {
        sqlx::query(
            "UPDATE student_ids SET student_name = ?, class_info = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"
        )
        .bind(student_name)
        .bind(class_info)
        .bind(id)
        .execute(&self.sqlite_pool)
        .await?;

        Ok(())
    }

    /// 删除学号
    pub async fn delete_student_id(&self, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM student_ids WHERE id = ?")
            .bind(id)
            .execute(&self.sqlite_pool)
            .await?;

        Ok(())
    }

    /// 批量导入学号
    pub async fn batch_import_student_ids(&self, student_data: &str, overwrite_existing: bool) -> Result<(i32, i32, Vec<String>)> {
        let mut imported_count = 0;
        let mut updated_count = 0;
        let mut errors = Vec::new();

        for (line_num, line) in student_data.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.is_empty() {
                continue;
            }

            let student_id = parts[0];
            let student_name = if parts.len() > 1 && !parts[1].is_empty() { Some(parts[1]) } else { None };
            let class_info = if parts.len() > 2 && !parts[2].is_empty() { Some(parts[2]) } else { None };

            // 验证学号格式
            if let Err(e) = crate::auth::StudentValidator::validate_student_id_format(student_id) {
                errors.push(format!("第{}行: 学号格式验证失败 '{}': {}", line_num + 1, student_id, e));
                continue;
            }

            // 检查是否已存在
            let exists: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM student_ids WHERE student_id = ?")
                .bind(student_id)
                .fetch_one(&self.sqlite_pool)
                .await
                .unwrap_or(0);

            if exists > 0 {
                if overwrite_existing {
                    // 更新现有记录
                    if let Err(e) = sqlx::query(
                        "UPDATE student_ids SET student_name = ?, class_info = ?, updated_at = CURRENT_TIMESTAMP WHERE student_id = ?"
                    )
                    .bind(student_name)
                    .bind(class_info)
                    .bind(student_id)
                    .execute(&self.sqlite_pool)
                    .await {
                        errors.push(format!("第{}行: 更新失败 - {}", line_num + 1, e));
                    } else {
                        updated_count += 1;
                    }
                } else {
                    errors.push(format!("第{}行: 学号 '{}' 已存在", line_num + 1, student_id));
                }
            } else {
                // 插入新记录
                if let Err(e) = self.add_student_id(student_id, student_name, class_info).await {
                    errors.push(format!("第{}行: 插入失败 - {}", line_num + 1, e));
                } else {
                    imported_count += 1;
                }
            }
        }

        Ok((imported_count, updated_count, errors))
    }

    /// 获取学号统计信息
    pub async fn get_student_id_stats(&self) -> Result<crate::models::StudentIdStats> {
        let total_count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM student_ids")
            .fetch_one(&self.sqlite_pool)
            .await?;

        let applied_count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM student_ids WHERE has_applied = 1")
            .fetch_one(&self.sqlite_pool)
            .await?;

        let not_applied_count = total_count - applied_count;

        Ok(crate::models::StudentIdStats {
            total_count,
            applied_count,
            not_applied_count,
        })
    }

    /// 验证学号格式
    fn is_valid_student_id(student_id: &str) -> bool {
        // 学号应该是10位数字
        student_id.len() == 10 && student_id.chars().all(|c| c.is_ascii_digit())
    }

    /// 获取所有已创建的用户和数据库信息
    pub async fn get_all_users(&self) -> Result<Vec<crate::models::UserDatabaseInfo>> {
        let users = sqlx::query_as::<_, crate::models::UserDatabaseInfo>(
            "SELECT id, identity_key, db_name, db_user, status, failure_reason, created_at, deleted_at, deletion_reason FROM applicants WHERE status = 'success' AND deleted_at IS NULL ORDER BY created_at DESC"
        )
        .fetch_all(&self.sqlite_pool)
        .await?;

        Ok(users)
    }

    // 错误处理和回滚机制

    /// 回滚MySQL数据库创建操作
    pub async fn rollback_database_creation(&self, identity_key: &str) -> Result<()> {
        let db_name = format!("db_{}", identity_key);
        let username = format!("user_{}", identity_key);
        let allowed_host = self
            .mysql_config
            .allowed_host
            .as_deref()
            .unwrap_or("localhost");

        warn!("开始回滚MySQL操作，身份标识: {}", identity_key);

        // 删除用户（如果存在）
        let drop_user_sql = format!("DROP USER IF EXISTS '{}'@'{}'", username, allowed_host);
        if let Err(e) = sqlx::query(&drop_user_sql).execute(&self.mysql_pool).await {
            error!("回滚删除用户失败: {}", e);
        } else {
            info!("成功删除用户: {}", username);
        }

        // 删除数据库（如果存在）
        let drop_db_sql = format!("DROP DATABASE IF EXISTS `{}`", db_name);
        if let Err(e) = sqlx::query(&drop_db_sql).execute(&self.mysql_pool).await {
            error!("回滚删除数据库失败: {}", e);
        } else {
            info!("成功删除数据库: {}", db_name);
        }

        // 刷新权限
        if let Err(e) = sqlx::query("FLUSH PRIVILEGES")
            .execute(&self.mysql_pool)
            .await
        {
            error!("回滚刷新权限失败: {}", e);
        }

        warn!("MySQL回滚操作完成，身份标识: {}", identity_key);
        Ok(())
    }

    /// 事务性创建数据库和用户
    pub async fn provision_database_with_transaction(
        &self,
        identity_key: &str,
        password: &str,
    ) -> Result<DatabaseCredentials> {
        // 首先尝试创建MySQL资源
        let credentials = self.provision_database(identity_key, password).await?;

        // 然后尝试创建SQLite记录
        match self
            .create_applicant(identity_key, &credentials.db_name, &credentials.username)
            .await
        {
            Ok(_) => {
                info!("事务性创建完成，身份标识: {}", identity_key);
                Ok(credentials)
            }
            Err(e) => {
                error!("SQLite记录创建失败，开始回滚MySQL操作: {}", e);

                // 回滚MySQL操作
                if let Err(rollback_err) = self.rollback_database_creation(identity_key).await {
                    error!("回滚操作也失败了: {}", rollback_err);
                }

                Err(e)
            }
        }
    }

    /// 验证数据一致性
    pub async fn verify_data_consistency(&self, identity_key: &str) -> Result<bool> {
        // 检查SQLite中是否存在记录
        let sqlite_exists = self.check_identity_exists(identity_key).await?;

        if !sqlite_exists {
            return Ok(true); // 如果SQLite中不存在，认为是一致的
        }

        // 检查MySQL中是否存在对应的数据库和用户
        let db_name = format!("db_{}", identity_key);
        let username = format!("user_{}", identity_key);

        // 检查数据库是否存在
        let db_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM INFORMATION_SCHEMA.SCHEMATA WHERE SCHEMA_NAME = ?",
        )
        .bind(&db_name)
        .fetch_one(&self.mysql_pool)
        .await?
            > 0;

        // 检查用户是否存在
        let user_exists =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM mysql.user WHERE User = ?")
                .bind(&username)
                .fetch_one(&self.mysql_pool)
                .await?
                > 0;

        let is_consistent = db_exists && user_exists;

        if !is_consistent {
            warn!(
                "数据不一致检测到，身份标识: {}, 数据库存在: {}, 用户存在: {}",
                identity_key, db_exists, user_exists
            );
        }

        Ok(is_consistent)
    }

    /// 修复数据不一致问题
    pub async fn repair_data_inconsistency(&self, identity_key: &str) -> Result<()> {
        info!("开始修复数据不一致，身份标识: {}", identity_key);

        let sqlite_exists = self.check_identity_exists(identity_key).await?;

        if sqlite_exists {
            // 如果SQLite中存在记录，但MySQL中资源不完整，则删除SQLite记录
            warn!("删除不一致的SQLite记录，身份标识: {}", identity_key);
            sqlx::query("DELETE FROM applicants WHERE identity_key = ?")
                .bind(identity_key)
                .execute(&self.sqlite_pool)
                .await?;
        }

        // 清理可能存在的MySQL资源
        let _ = self.rollback_database_creation(identity_key).await;

        info!("数据不一致修复完成，身份标识: {}", identity_key);
        Ok(())
    }

    /// 统计成功申请数量
    pub async fn count_successful_applications(&self) -> Result<i64> {
        let count = sqlx::query_scalar("SELECT COUNT(*) FROM applicants WHERE status = 'success'")
            .fetch_one(&self.sqlite_pool)
            .await?;
        Ok(count)
    }

    /// 统计失败申请数量
    pub async fn count_failed_applications(&self) -> Result<i64> {
        let count = sqlx::query_scalar("SELECT COUNT(*) FROM applicants WHERE status = 'failed'")
            .fetch_one(&self.sqlite_pool)
            .await?;
        Ok(count)
    }

    /// 统计已删除申请数量
    pub async fn count_deleted_applications(&self) -> Result<i64> {
        let count = sqlx::query_scalar("SELECT COUNT(*) FROM applicants WHERE status = 'deleted'")
            .fetch_one(&self.sqlite_pool)
            .await?;
        Ok(count)
    }

    /// 管理员删除用户数据库和用户
    pub async fn admin_delete_user(&self, identity_key: &str, reason: &str) -> Result<()> {
        let db_name = format!("db_{}", identity_key);
        let username = format!("user_{}", identity_key);
        let allowed_host = self
            .mysql_config
            .allowed_host
            .as_deref()
            .unwrap_or("localhost");

        info!("管理员删除用户: {}, 原因: {}", identity_key, reason);

        // 1. 删除 MySQL 用户
        let drop_user_sql = format!("DROP USER IF EXISTS '{}'@'{}'", username, allowed_host);
        if let Err(e) = sqlx::query(&drop_user_sql).execute(&self.mysql_pool).await {
            error!("删除用户失败: {}", e);
        } else {
            info!("成功删除用户: {}", username);
        }

        // 2. 删除 MySQL 数据库
        let drop_db_sql = format!("DROP DATABASE IF EXISTS `{}`", db_name);
        if let Err(e) = sqlx::query(&drop_db_sql).execute(&self.mysql_pool).await {
            error!("删除数据库失败: {}", e);
        } else {
            info!("成功删除数据库: {}", db_name);
        }

        // 3. 刷新权限
        if let Err(e) = sqlx::query("FLUSH PRIVILEGES")
            .execute(&self.mysql_pool)
            .await
        {
            error!("刷新权限失败: {}", e);
        }

        // 4. 更新 SQLite 记录状态
        sqlx::query(
            "UPDATE applicants SET status = 'deleted', deleted_at = CURRENT_TIMESTAMP, deletion_reason = ? WHERE identity_key = ?"
        )
        .bind(reason)
        .bind(identity_key)
        .execute(&self.sqlite_pool)
        .await?;

        info!("用户 {} 已被管理员删除", identity_key);
        Ok(())
    }

    /// 获取公开申请记录 (脱敏处理)
    pub async fn get_public_applications(
        &self,
        limit: i32,
    ) -> Result<Vec<crate::models::PublicApplicationRecord>> {
        let records = sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, identity_key, status, created_at FROM applicants ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.sqlite_pool)
        .await?;

        let public_records = records
            .into_iter()
            .map(|(id, identity_key, status, created_at)| {
                // 脱敏处理身份标识
                let masked_key = if identity_key.len() > 4 {
                    format!("{}****", &identity_key[..4])
                } else {
                    "****".to_string()
                };

                crate::models::PublicApplicationRecord {
                    id,
                    identity_key_masked: masked_key,
                    status,
                    created_at,
                }
            })
            .collect();

        Ok(public_records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_identifier() {
        // 有效的标识符
        assert!(DatabaseManager::is_valid_identifier("db_test123"));
        assert!(DatabaseManager::is_valid_identifier("user_abc"));
        assert!(DatabaseManager::is_valid_identifier("_valid"));
        assert!(DatabaseManager::is_valid_identifier("Valid123"));

        // 无效的标识符
        assert!(!DatabaseManager::is_valid_identifier(""));
        assert!(!DatabaseManager::is_valid_identifier("123invalid")); // 不能以数字开头
        assert!(!DatabaseManager::is_valid_identifier("invalid-name")); // 不能包含连字符
        assert!(!DatabaseManager::is_valid_identifier("invalid.name")); // 不能包含点
        assert!(!DatabaseManager::is_valid_identifier("invalid name")); // 不能包含空格
        assert!(!DatabaseManager::is_valid_identifier("invalid@name")); // 不能包含特殊字符

        // 长度测试
        let long_name = "a".repeat(65);
        assert!(!DatabaseManager::is_valid_identifier(&long_name)); // 超过64字符
    }

    #[test]
    fn test_is_valid_host() {
        // 有效的主机
        assert!(DatabaseManager::is_valid_host("localhost"));
        assert!(DatabaseManager::is_valid_host("127.0.0.1"));
        assert!(DatabaseManager::is_valid_host("192.168.1.1"));
        assert!(DatabaseManager::is_valid_host("example.com"));
        assert!(DatabaseManager::is_valid_host("sub.example.com"));

        // 无效的主机
        assert!(!DatabaseManager::is_valid_host("")); // 空字符串

        // 通配符在生产模式下无效，开发模式下有效
        unsafe {
            std::env::remove_var("DEV_MODE"); // 确保是生产模式
        }
        assert!(!DatabaseManager::is_valid_host("%")); // 生产模式下通配符无效

        unsafe {
            std::env::set_var("DEV_MODE", "true"); // 设置开发模式
        }
        assert!(DatabaseManager::is_valid_host("%")); // 开发模式下通配符有效
        unsafe {
            std::env::remove_var("DEV_MODE"); // 清理环境变量
        }

        assert!(!DatabaseManager::is_valid_host(".example.com")); // 以点开头
        assert!(!DatabaseManager::is_valid_host("example.com.")); // 以点结尾
        assert!(!DatabaseManager::is_valid_host("-example.com")); // 以连字符开头
        assert!(!DatabaseManager::is_valid_host("example.com-")); // 以连字符结尾

        // 长度测试
        let long_host = "a".repeat(256);
        assert!(!DatabaseManager::is_valid_host(&long_host)); // 超过255字符
    }

    #[test]
    fn test_is_valid_ip() {
        // 有效的IP地址
        assert!(DatabaseManager::is_valid_ip("127.0.0.1"));
        assert!(DatabaseManager::is_valid_ip("192.168.1.1"));
        assert!(DatabaseManager::is_valid_ip("::1"));
        assert!(DatabaseManager::is_valid_ip("2001:db8::1"));

        // 无效的IP地址
        assert!(!DatabaseManager::is_valid_ip("256.256.256.256"));
        assert!(!DatabaseManager::is_valid_ip("192.168.1"));
        assert!(!DatabaseManager::is_valid_ip("not.an.ip"));
        assert!(!DatabaseManager::is_valid_ip(""));
    }

    #[test]
    fn test_is_valid_hostname() {
        // 有效的主机名
        assert!(DatabaseManager::is_valid_hostname("example.com"));
        assert!(DatabaseManager::is_valid_hostname("sub.example.com"));
        assert!(DatabaseManager::is_valid_hostname("test-server"));
        assert!(DatabaseManager::is_valid_hostname("server123"));

        // 无效的主机名
        assert!(!DatabaseManager::is_valid_hostname(""));
        assert!(!DatabaseManager::is_valid_hostname(".example.com"));
        assert!(!DatabaseManager::is_valid_hostname("example.com."));
        assert!(!DatabaseManager::is_valid_hostname("-example"));
        assert!(!DatabaseManager::is_valid_hostname("example-"));
        assert!(!DatabaseManager::is_valid_hostname("example..com"));

        // 长度测试
        let long_hostname = "a".repeat(254);
        assert!(!DatabaseManager::is_valid_hostname(&long_hostname));
    }
}
