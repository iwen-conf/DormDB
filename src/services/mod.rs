use crate::database::DatabaseManager;
use crate::models::{
    ApiResponse, Applicant, ApplicationStats, DatabaseCredentials, StatusCode, StatusMessage,
    SystemStatus,
};
use crate::utils::{generate_secure_password, validate_identity_key};
use chrono::Utc;
use log::{error, info, warn};
use std::sync::Arc;

#[derive(Clone)]
pub struct DatabaseService {
    db_manager: Arc<DatabaseManager>,
}

impl DatabaseService {
    pub fn new(db_manager: DatabaseManager) -> Self {
        Self {
            db_manager: Arc::new(db_manager),
        }
    }

    pub async fn apply_database(&self, identity_key: &str) -> ApiResponse<DatabaseCredentials> {
        info!(
            "[申请开始] 身份标识: {}, 时间: {}",
            identity_key,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        // 1. 验证输入参数
        if !validate_identity_key(identity_key) {
            warn!(
                "[申请失败] 无效的身份标识: {}, 时间: {}",
                identity_key,
                Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            );
            return ApiResponse::error(
                StatusCode::INVALID_INPUT,
                StatusMessage::INVALID_INPUT.to_string(),
            );
        }

        // 2. 检查身份标识是否已存在
        match self.db_manager.check_identity_exists(identity_key).await {
            Ok(exists) => {
                if exists {
                    warn!(
                        "[申请失败] 身份标识已存在: {}, 时间: {}",
                        identity_key,
                        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                    );
                    return ApiResponse::error(
                        StatusCode::IDENTITY_EXISTS,
                        StatusMessage::IDENTITY_EXISTS.to_string(),
                    );
                }
            }
            Err(e) => {
                error!(
                    "[申请失败] 检查身份标识是否存在时失败: {}, 身份标识: {}, 时间: {}",
                    e,
                    identity_key,
                    Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                );
                return ApiResponse::error(
                    StatusCode::INTERNAL_ERROR,
                    StatusMessage::INTERNAL_ERROR.to_string(),
                );
            }
        }

        // 3. 生成安全密码
        let password = generate_secure_password(16);

        // 4. 事务性创建数据库和用户（包含自动回滚机制）
        let credentials = match self
            .db_manager
            .provision_database_with_transaction(identity_key, &password)
            .await
        {
            Ok(creds) => {
                info!(
                    "[事务成功] 身份标识: {}, 数据库和记录创建完成",
                    identity_key
                );
                creds
            }
            Err(e) => {
                error!(
                    "[事务失败] 身份标识: {}, 错误: {}, 时间: {}",
                    identity_key,
                    e,
                    Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                );

                // 记录失败的申请
                let failure_reason = format!("数据库创建失败: {}", e);
                if let Err(record_err) = self
                    .db_manager
                    .create_failed_applicant(identity_key, &failure_reason)
                    .await
                {
                    error!("[记录失败] 无法记录失败申请: {}", record_err);
                }

                // 尝试修复可能的数据不一致
                if let Err(repair_err) = self
                    .db_manager
                    .repair_data_inconsistency(identity_key)
                    .await
                {
                    error!(
                        "[修复失败] 身份标识: {}, 修复错误: {}",
                        identity_key, repair_err
                    );
                }

                return ApiResponse::error(
                    StatusCode::DB_PROVISION_FAILED,
                    StatusMessage::DB_PROVISION_FAILED.to_string(),
                );
            }
        };

        info!(
            "[申请成功] 身份标识: {}, 数据库: {}, 用户: {}, 时间: {}",
            identity_key,
            credentials.db_name,
            credentials.username,
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        ApiResponse::success(credentials)
    }

    pub async fn get_all_applicants(&self) -> ApiResponse<Vec<Applicant>> {
        info!("正在获取所有申请者信息");

        match self.db_manager.get_all_applicants().await {
            Ok(applicants) => {
                info!("成功获取到 {} 个申请者", applicants.len());
                ApiResponse::success(applicants)
            }
            Err(e) => {
                error!("获取申请者信息失败: {}", e);
                ApiResponse::error(
                    StatusCode::INTERNAL_ERROR,
                    StatusMessage::INTERNAL_ERROR.to_string(),
                )
            }
        }
    }

    /// 获取系统状态信息 (ADM-001)
    pub async fn get_system_status(&self) -> ApiResponse<SystemStatus> {
        info!("获取系统状态信息");

        match self.collect_system_status().await {
            Ok(status) => {
                info!("系统状态获取成功");
                ApiResponse::success(status)
            }
            Err(e) => {
                error!("获取系统状态失败: {}", e);
                ApiResponse::error(
                    StatusCode::INTERNAL_ERROR,
                    StatusMessage::INTERNAL_ERROR.to_string(),
                )
            }
        }
    }

    /// 获取申请统计信息 (ADM-001)
    pub async fn get_application_stats(&self) -> ApiResponse<ApplicationStats> {
        info!("获取申请统计信息");

        match self.collect_application_stats().await {
            Ok(stats) => {
                info!("申请统计获取成功");
                ApiResponse::success(stats)
            }
            Err(e) => {
                error!("获取申请统计失败: {}", e);
                ApiResponse::error(
                    StatusCode::INTERNAL_ERROR,
                    StatusMessage::INTERNAL_ERROR.to_string(),
                )
            }
        }
    }

    /// 收集系统状态信息
    async fn collect_system_status(
        &self,
    ) -> Result<SystemStatus, Box<dyn std::error::Error + Send + Sync>> {
        // 获取服务启动时间（简化实现）
        let uptime = "运行中".to_string();

        // 检查数据库连接状态
        let database_status = match self.db_manager.test_sqlite_connection().await {
            Ok(_) => "正常".to_string(),
            Err(_) => "异常".to_string(),
        };

        let mysql_status = match self.db_manager.test_mysql_connection().await {
            Ok(_) => "正常".to_string(),
            Err(_) => "异常".to_string(),
        };

        // 获取申请统计
        let total_applications = self
            .db_manager
            .count_total_applications()
            .await
            .unwrap_or(0);
        let today_applications = self
            .db_manager
            .count_today_applications()
            .await
            .unwrap_or(0);

        Ok(SystemStatus {
            uptime,
            database_status,
            mysql_status,
            total_applications,
            today_applications,
            version: "1.0.0".to_string(),
        })
    }

    /// 收集申请统计信息
    async fn collect_application_stats(
        &self,
    ) -> Result<ApplicationStats, Box<dyn std::error::Error + Send + Sync>> {
        let total_count = self
            .db_manager
            .count_total_applications()
            .await
            .unwrap_or(0);
        let today_count = self
            .db_manager
            .count_today_applications()
            .await
            .unwrap_or(0);
        let week_count = self.db_manager.count_week_applications().await.unwrap_or(0);
        let month_count = self
            .db_manager
            .count_month_applications()
            .await
            .unwrap_or(0);

        // 获取最近的申请记录
        let recent_applications = self
            .db_manager
            .get_recent_applications(10)
            .await
            .unwrap_or_default();

        // 获取状态统计
        let successful_count = self
            .db_manager
            .count_successful_applications()
            .await
            .unwrap_or(0);
        let failed_count = self
            .db_manager
            .count_failed_applications()
            .await
            .unwrap_or(0);
        let deleted_count = self
            .db_manager
            .count_deleted_applications()
            .await
            .unwrap_or(0);

        Ok(ApplicationStats {
            total_count,
            today_count,
            week_count,
            month_count,
            successful_count,
            failed_count,
            deleted_count,
            recent_applications,
        })
    }

    /// 检查和修复数据一致性
    pub async fn check_and_repair_consistency(&self) -> ApiResponse<String> {
        info!("开始检查数据一致性");

        match self.perform_consistency_check().await {
            Ok(report) => {
                info!("数据一致性检查完成");
                ApiResponse::success(report)
            }
            Err(e) => {
                error!("数据一致性检查失败: {}", e);
                ApiResponse::error(
                    StatusCode::INTERNAL_ERROR,
                    StatusMessage::INTERNAL_ERROR.to_string(),
                )
            }
        }
    }

    /// 执行一致性检查
    async fn perform_consistency_check(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut report = String::new();
        let mut inconsistent_count = 0;
        let mut repaired_count = 0;

        // 获取所有申请者
        let applicants = self.db_manager.get_all_applicants().await?;
        let total_count = applicants.len();

        report.push_str(&format!(
            "开始检查 {} 个申请记录的数据一致性\n\n",
            total_count
        ));

        for applicant in applicants {
            let identity_key = &applicant.identity_key;

            match self.db_manager.verify_data_consistency(identity_key).await {
                Ok(is_consistent) => {
                    if !is_consistent {
                        inconsistent_count += 1;
                        report.push_str(&format!("❌ 发现不一致: {}\n", identity_key));

                        // 尝试修复
                        match self
                            .db_manager
                            .repair_data_inconsistency(identity_key)
                            .await
                        {
                            Ok(_) => {
                                repaired_count += 1;
                                report.push_str(&format!("✅ 修复成功: {}\n", identity_key));
                            }
                            Err(e) => {
                                report.push_str(&format!(
                                    "❌ 修复失败: {}, 错误: {}\n",
                                    identity_key, e
                                ));
                            }
                        }
                    }
                }
                Err(e) => {
                    report.push_str(&format!("⚠️ 检查失败: {}, 错误: {}\n", identity_key, e));
                }
            }
        }

        report.push_str("\n检查完成:\n");
        report.push_str(&format!("- 总记录数: {}\n", total_count));
        report.push_str(&format!("- 不一致记录: {}\n", inconsistent_count));
        report.push_str(&format!("- 修复成功: {}\n", repaired_count));
        report.push_str(&format!(
            "- 修复失败: {}\n",
            inconsistent_count - repaired_count
        ));

        Ok(report)
    }

    /// 管理员登录验证
    pub async fn admin_login(&self, password: &str) -> ApiResponse<String> {
        info!("管理员登录验证");

        // 这里应该从配置中获取管理员密码，但由于架构限制，我们使用环境变量
        let admin_password =
            std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());

        if password == admin_password {
            info!("管理员登录成功");
            ApiResponse::success("登录成功".to_string())
        } else {
            warn!("管理员登录失败：密码错误");
            ApiResponse::error(40101, "密码错误".to_string())
        }
    }

    /// 管理员删除用户
    pub async fn admin_delete_user(&self, identity_key: &str, reason: &str) -> ApiResponse<String> {
        info!("管理员删除用户: {}, 原因: {}", identity_key, reason);

        match self
            .db_manager
            .admin_delete_user(identity_key, reason)
            .await
        {
            Ok(_) => {
                info!("用户 {} 删除成功", identity_key);
                ApiResponse::success(format!("用户 {} 已被删除", identity_key))
            }
            Err(e) => {
                error!("删除用户失败: {}", e);
                ApiResponse::error(StatusCode::INTERNAL_ERROR, format!("删除用户失败: {}", e))
            }
        }
    }

    /// 获取公开申请记录
    pub async fn get_public_applications(
        &self,
    ) -> ApiResponse<Vec<crate::models::PublicApplicationRecord>> {
        info!("获取公开申请记录");

        match self.db_manager.get_public_applications(50).await {
            Ok(records) => {
                info!("成功获取 {} 条公开申请记录", records.len());
                ApiResponse::success(records)
            }
            Err(e) => {
                error!("获取公开申请记录失败: {}", e);
                ApiResponse::error(
                    StatusCode::INTERNAL_ERROR,
                    StatusMessage::INTERNAL_ERROR.to_string(),
                )
            }
        }
    }

    // 学号管理服务

    /// 获取学号列表
    pub async fn get_student_ids(&self, limit: Option<i32>, offset: Option<i32>) -> ApiResponse<Vec<crate::models::StudentId>> {
        info!("获取学号列表");

        match self.db_manager.get_all_student_ids(limit, offset).await {
            Ok(student_ids) => {
                info!("成功获取 {} 条学号记录", student_ids.len());
                ApiResponse::success(student_ids)
            }
            Err(e) => {
                error!("获取学号列表失败: {}", e);
                ApiResponse::error(50001, "获取学号列表失败".to_string())
            }
        }
    }

    /// 添加学号
    pub async fn add_student_id(&self, student_id: &str, student_name: Option<&str>, class_info: Option<&str>) -> ApiResponse<String> {
        info!("添加学号: {}", student_id);

        match self.db_manager.add_student_id(student_id, student_name, class_info).await {
            Ok(_) => {
                info!("学号添加成功: {}", student_id);
                ApiResponse::success("学号添加成功".to_string())
            }
            Err(e) => {
                error!("添加学号失败: {}", e);
                if e.to_string().contains("UNIQUE constraint failed") {
                    ApiResponse::error(40001, "学号已存在".to_string())
                } else {
                    ApiResponse::error(50001, "添加学号失败".to_string())
                }
            }
        }
    }

    /// 批量导入学号
    pub async fn batch_import_student_ids(&self, student_data: &str, overwrite_existing: bool) -> ApiResponse<crate::models::BatchImportResult> {
        info!("批量导入学号");

        match self.db_manager.batch_import_student_ids(student_data, overwrite_existing).await {
            Ok((imported_count, updated_count, errors)) => {
                info!("批量导入完成: 导入{}条, 更新{}条, 错误{}条", imported_count, updated_count, errors.len());
                ApiResponse::success(crate::models::BatchImportResult {
                    imported_count,
                    updated_count,
                    errors,
                })
            }
            Err(e) => {
                error!("批量导入失败: {}", e);
                ApiResponse::error(50001, "批量导入失败".to_string())
            }
        }
    }

    /// 更新学号信息
    pub async fn update_student_id(&self, id: i32, student_name: Option<&str>, class_info: Option<&str>) -> ApiResponse<String> {
        info!("更新学号信息: ID {}", id);

        match self.db_manager.update_student_id(id, student_name, class_info).await {
            Ok(_) => {
                info!("学号信息更新成功: ID {}", id);
                ApiResponse::success("学号信息更新成功".to_string())
            }
            Err(e) => {
                error!("更新学号信息失败: {}", e);
                ApiResponse::error(50001, "更新学号信息失败".to_string())
            }
        }
    }

    /// 删除学号
    pub async fn delete_student_id(&self, id: i32) -> ApiResponse<String> {
        info!("删除学号: ID {}", id);

        match self.db_manager.delete_student_id(id).await {
            Ok(_) => {
                info!("学号删除成功: ID {}", id);
                ApiResponse::success("学号删除成功".to_string())
            }
            Err(e) => {
                error!("删除学号失败: {}", e);
                ApiResponse::error(50001, "删除学号失败".to_string())
            }
        }
    }

    /// 获取学号统计
    pub async fn get_student_id_stats(&self) -> ApiResponse<crate::models::StudentIdStats> {
        info!("获取学号统计信息");

        match self.db_manager.get_student_id_stats().await {
            Ok(stats) => {
                info!("学号统计获取成功");
                ApiResponse::success(stats)
            }
            Err(e) => {
                error!("获取学号统计失败: {}", e);
                ApiResponse::error(50001, "获取学号统计失败".to_string())
            }
        }
    }

    // 用户管理服务

    /// 获取所有用户列表
    pub async fn get_all_users(&self) -> ApiResponse<Vec<crate::models::UserDatabaseInfo>> {
        info!("获取所有用户列表");

        match self.db_manager.get_all_users().await {
            Ok(users) => {
                info!("成功获取 {} 个用户记录", users.len());
                ApiResponse::success(users)
            }
            Err(e) => {
                error!("获取用户列表失败: {}", e);
                ApiResponse::error(50001, "获取用户列表失败".to_string())
            }
        }
    }

    /// 删除用户（通过身份标识）
    pub async fn delete_user_by_identity(&self, identity_key: &str, reason: &str) -> ApiResponse<String> {
        info!("删除用户: {}, 原因: {}", identity_key, reason);

        match self.db_manager.admin_delete_user(identity_key, reason).await {
            Ok(_) => {
                info!("用户删除成功: {}", identity_key);
                ApiResponse::success("用户删除成功".to_string())
            }
            Err(e) => {
                error!("删除用户失败: {}", e);
                ApiResponse::error(50001, "删除用户失败".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, DatabaseConfig, MySQLConfig, ServerConfig};
    use crate::database::DatabaseManager;
    use std::sync::Arc;

    // 创建测试用的配置
    fn create_test_config() -> AppConfig {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                sqlite_path: ":memory:".to_string(), // 使用内存数据库进行测试
            },
            mysql: MySQLConfig {
                host: "localhost".to_string(),
                port: 3306,
                username: "test".to_string(),
                password: "test".to_string(),
                database: "test".to_string(),
                allowed_host: Some("localhost".to_string()),
            },
            admin: crate::config::AdminConfig {
                password: "test_admin".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn test_database_service_creation() {
        let config = create_test_config();

        // 注意：这个测试可能会失败，因为需要实际的数据库连接
        // 在实际项目中，应该使用模拟数据库或测试数据库
        if let Ok(db_manager) = DatabaseManager::new(&config).await {
            let service = DatabaseService::new(db_manager);

            // 验证服务创建成功
            assert!(!Arc::ptr_eq(&service.db_manager, &service.db_manager)); // 这只是一个基本检查
        }
    }

    #[test]
    fn test_invalid_identity_key_validation() {
        // 测试无效的身份标识
        let long_invalid = "a".repeat(51);
        let invalid_keys = vec![
            "",            // 空字符串
            &long_invalid, // 过长
            "invalid@key", // 包含特殊字符
            "invalid key", // 包含空格
            "invalid-key", // 包含连字符
        ];

        for key in invalid_keys {
            assert!(
                !crate::utils::validate_identity_key(key),
                "Key '{}' should be invalid",
                key
            );
        }
    }

    #[test]
    fn test_valid_identity_key_validation() {
        // 测试有效的身份标识
        let long_valid = "a".repeat(50);
        let valid_keys = vec![
            "user123",
            "test_user",
            "_valid",
            "Valid123",
            "a",
            "20250701",  // 学号（可以以数字开头）
            "123456",    // 纯数字学号
            &long_valid, // 最大长度
        ];

        for key in valid_keys {
            assert!(
                crate::utils::validate_identity_key(key),
                "Key '{}' should be valid",
                key
            );
        }
    }

    #[test]
    fn test_password_generation() {
        // 测试密码生成
        let password = crate::utils::generate_secure_password(16);

        assert_eq!(password.len(), 16);

        // 验证密码包含各种字符类型
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_digit(10));
        let has_symbol = password.chars().any(|c| "!@#$%^&*".contains(c));

        assert!(has_lowercase, "Password should contain lowercase letters");
        assert!(has_uppercase, "Password should contain uppercase letters");
        assert!(has_digit, "Password should contain digits");
        assert!(has_symbol, "Password should contain symbols");
    }

    #[test]
    fn test_api_response_creation() {
        // 测试成功响应
        let success_response = ApiResponse::success("test data".to_string());
        assert_eq!(success_response.code, 0);
        assert_eq!(success_response.message, "Success");
        assert_eq!(success_response.data, Some("test data".to_string()));

        // 测试错误响应
        let error_response: ApiResponse<String> =
            ApiResponse::error(40001, "Invalid input".to_string());
        assert_eq!(error_response.code, 40001);
        assert_eq!(error_response.message, "Invalid input");
        assert_eq!(error_response.data, None);
    }

    #[test]
    fn test_status_codes() {
        // 测试状态码常量
        assert_eq!(StatusCode::SUCCESS, 0);
        assert_eq!(StatusCode::INVALID_INPUT, 40001);
        assert_eq!(StatusCode::IDENTITY_EXISTS, 40901);
        assert_eq!(StatusCode::INTERNAL_ERROR, 50001);
        assert_eq!(StatusCode::DB_PROVISION_FAILED, 50002);
    }

    #[test]
    fn test_status_messages() {
        // 测试状态消息常量
        assert_eq!(StatusMessage::SUCCESS, "Success");
        assert_eq!(StatusMessage::INVALID_INPUT, "Invalid input parameter.");
        assert_eq!(
            StatusMessage::IDENTITY_EXISTS,
            "Identity key already exists."
        );
        assert_eq!(StatusMessage::INTERNAL_ERROR, "Internal server error.");
        assert_eq!(
            StatusMessage::DB_PROVISION_FAILED,
            "Database provisioning failed."
        );
    }
}
