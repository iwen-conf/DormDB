use anyhow::{Result, anyhow};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub mysql: MySQLConfig,
    pub admin: AdminConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub sqlite_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySQLConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub allowed_host: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminConfig {
    pub password: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        info!("开始加载配置...");

        // 服务器配置
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| {
            info!("使用默认服务器地址: 127.0.0.1");
            "127.0.0.1".to_string()
        });

        let server_port = match env::var("SERVER_PORT") {
            Ok(port_str) => match port_str.parse::<u16>() {
                Ok(port) => {
                    if port < 1024 {
                        warn!("端口号 {} 小于 1024，可能需要管理员权限", port);
                    }
                    port
                }
                Err(_) => {
                    warn!("无效的端口号 '{}', 使用默认端口 3000", port_str);
                    3000
                }
            },
            Err(_) => {
                info!("使用默认端口: 3000");
                3000
            }
        };

        // 数据库配置
        let sqlite_path = env::var("SQLITE_PATH").unwrap_or_else(|_| {
            info!("使用默认SQLite路径: ./dormdb_state.db");
            "./dormdb_state.db".to_string()
        });

        // MySQL 配置
        let mysql_host = env::var("MYSQL_HOST").unwrap_or_else(|_| {
            info!("使用默认MySQL主机: sql.iluwen.cn");
            "sql.iluwen.cn".to_string()
        });

        let mysql_port = match env::var("MYSQL_PORT") {
            Ok(port_str) => match port_str.parse::<u16>() {
                Ok(port) => port,
                Err(_) => {
                    warn!("无效的MySQL端口号 '{}', 使用默认端口 49500", port_str);
                    49500
                }
            },
            Err(_) => {
                info!("使用默认MySQL端口: 49500");
                49500
            }
        };

        let mysql_username = env::var("MYSQL_USERNAME").unwrap_or_else(|_| {
            info!("使用默认MySQL用户名: kaiwen");
            "kaiwen".to_string()
        });

        let mysql_password =
            env::var("MYSQL_PASSWORD").map_err(|_| anyhow!("MYSQL_PASSWORD 环境变量是必需的"))?;

        let mysql_database = env::var("MYSQL_DATABASE").unwrap_or_else(|_| {
            info!("使用默认MySQL数据库: default");
            "default".to_string()
        });

        let mysql_allowed_host = env::var("MYSQL_ALLOWED_HOST").ok();
        if let Some(ref host) = mysql_allowed_host {
            // 检查是否是开发模式
            let is_dev_mode = env::var("DEV_MODE").unwrap_or_default() == "true";

            if host == "%" && !is_dev_mode {
                return Err(anyhow!(
                    "安全错误: MYSQL_ALLOWED_HOST 不能设置为通配符 '%'，除非设置 DEV_MODE=true"
                )
                .into());
            }

            if host == "%" && is_dev_mode {
                warn!("⚠️  开发模式: 允许通配符主机 '%' - 仅用于开发环境！");
            }

            info!("MySQL允许的主机: {}", host);
        } else {
            info!("MySQL允许的主机: localhost (默认)");
        }

        // 管理员配置
        let admin_password = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| {
            warn!("未设置 ADMIN_PASSWORD，使用默认密码 'admin123'");
            "admin123".to_string()
        });

        if admin_password == "admin123" {
            warn!("⚠️  警告: 正在使用默认管理员密码，请在生产环境中修改！");
        }

        let config = AppConfig {
            server: ServerConfig {
                host: server_host,
                port: server_port,
            },
            database: DatabaseConfig { sqlite_path },
            mysql: MySQLConfig {
                host: mysql_host,
                port: mysql_port,
                username: mysql_username,
                password: mysql_password,
                database: mysql_database,
                allowed_host: mysql_allowed_host,
            },
            admin: AdminConfig {
                password: admin_password,
            },
        };

        // 验证配置
        config.validate()?;

        info!("配置加载完成");
        Ok(config)
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 验证服务器配置
        if self.server.host.is_empty() {
            return Err(anyhow!("服务器地址不能为空").into());
        }

        if self.server.port == 0 {
            return Err(anyhow!("服务器端口不能为0").into());
        }

        // 验证数据库配置
        if self.database.sqlite_path.is_empty() {
            return Err(anyhow!("SQLite数据库路径不能为空").into());
        }

        // 验证MySQL配置
        if self.mysql.host.is_empty() {
            return Err(anyhow!("MySQL主机地址不能为空").into());
        }

        if self.mysql.port == 0 {
            return Err(anyhow!("MySQL端口不能为0").into());
        }

        if self.mysql.username.is_empty() {
            return Err(anyhow!("MySQL用户名不能为空").into());
        }

        if self.mysql.password.is_empty() {
            return Err(anyhow!("MySQL密码不能为空").into());
        }

        if self.mysql.database.is_empty() {
            return Err(anyhow!("MySQL数据库名不能为空").into());
        }

        // 验证允许的主机配置
        if let Some(ref allowed_host) = self.mysql.allowed_host {
            // 检查开发模式
            let is_dev_mode = std::env::var("DEV_MODE").unwrap_or_default() == "true";

            if allowed_host == "%" && !is_dev_mode {
                return Err(anyhow!(
                    "安全错误: 生产环境不允许使用通配符主机 '%'，请设置 DEV_MODE=true 或使用具体IP"
                )
                .into());
            }

            if allowed_host.is_empty() {
                return Err(anyhow!("允许的主机不能为空字符串").into());
            }
        }

        info!("配置验证通过");
        Ok(())
    }

    /// 显示配置摘要（隐藏敏感信息）
    pub fn display_summary(&self) {
        info!("=== DormDB 配置摘要 ===");
        info!("服务器: {}:{}", self.server.host, self.server.port);
        info!("SQLite: {}", self.database.sqlite_path);
        info!(
            "MySQL: {}:{}/{}",
            self.mysql.host, self.mysql.port, self.mysql.database
        );
        info!("MySQL用户: {}", self.mysql.username);
        info!("MySQL密码: [已设置]");
        if let Some(ref host) = self.mysql.allowed_host {
            info!("允许的主机: {}", host);
        } else {
            info!("允许的主机: localhost (默认)");
        }
        info!("========================");
    }
}
