use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// 数据库申请请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApplyRequest {
    /// 用户身份标识，如学号
    #[schema(example = "20250701")]
    pub identity_key: String,
}

/// 统一API响应结构
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
    /// 业务状态码，0表示成功，非0表示失败
    #[schema(example = 0)]
    pub code: i32,
    /// 响应消息
    #[schema(example = "Success")]
    pub message: String,
    /// 响应数据
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "Success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }
}

/// 数据库申请成功后的响应数据
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DatabaseCredentials {
    /// 数据库主机地址
    #[schema(example = "sql.iluwen.cn")]
    pub db_host: String,
    /// 数据库端口
    #[schema(example = 49500)]
    pub db_port: u16,
    /// 数据库名称
    #[schema(example = "db_20250701")]
    pub db_name: String,
    /// 数据库用户名
    #[schema(example = "user_20250701")]
    pub username: String,
    /// 数据库密码
    #[schema(example = "GeneratedSecurePassword123")]
    pub password: String,
    /// 完整的连接字符串 (包含必要参数)
    #[schema(
        example = "mysql://user_20250701:GeneratedSecurePassword123@sql.iluwen.cn:49500/db_20250701?allowPublicKeyRetrieval=true&useSSL=false"
    )]
    pub connection_string: String,
    /// JDBC 连接字符串 (Java 应用使用)
    #[schema(
        example = "jdbc:mysql://sql.iluwen.cn:49500/db_20250701?allowPublicKeyRetrieval=true&useSSL=false&user=user_20250701&password=GeneratedSecurePassword123"
    )]
    pub jdbc_url: String,
}

/// 学号管理记录
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct StudentId {
    /// 记录ID
    #[schema(example = 1)]
    pub id: i32,
    /// 学号
    #[schema(example = "2203010301")]
    pub student_id: String,
    /// 学生姓名（可选）
    #[schema(example = "张三")]
    pub student_name: Option<String>,
    /// 专业班级（可选）
    #[schema(example = "计算机科学与技术2022级3班")]
    pub class_info: Option<String>,
    /// 是否已申请数据库
    #[schema(example = false)]
    pub has_applied: bool,
    /// 申请的数据库名（如果已申请）
    #[schema(example = "db_2203010301")]
    pub applied_db_name: Option<String>,
    /// 创建时间
    #[schema(example = "2025-07-14T10:00:00Z")]
    pub created_at: String,
    /// 更新时间
    #[schema(example = "2025-07-14T10:00:00Z")]
    pub updated_at: String,
}

/// 学号批量导入请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StudentIdBatchImport {
    /// 学号列表，每行一个学号，格式：学号,姓名,班级（姓名和班级可选）
    #[schema(example = "2203010301,张三,计算机2022-3班\n2203010302,李四,计算机2022-3班")]
    pub student_data: String,
    /// 是否覆盖已存在的学号
    #[schema(example = false)]
    pub overwrite_existing: bool,
}

/// 学号管理统计
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StudentIdStats {
    /// 总学号数
    #[schema(example = 150)]
    pub total_count: i32,
    /// 已申请数据库的学号数
    #[schema(example = 45)]
    pub applied_count: i32,
    /// 未申请数据库的学号数
    #[schema(example = 105)]
    pub not_applied_count: i32,
}

/// 分页查询参数
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginationQuery {
    /// 每页数量
    #[schema(example = 100)]
    pub limit: Option<i32>,
    /// 偏移量
    #[schema(example = 0)]
    pub offset: Option<i32>,
}

/// 添加学号请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddStudentIdRequest {
    /// 学号
    #[schema(example = "2203010301")]
    pub student_id: String,
    /// 学生姓名（可选）
    #[schema(example = "张三")]
    pub student_name: Option<String>,
    /// 专业班级（可选）
    #[schema(example = "计算机科学与技术2022级3班")]
    pub class_info: Option<String>,
}

/// 更新学号请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateStudentIdRequest {
    /// 学生姓名（可选）
    #[schema(example = "张三")]
    pub student_name: Option<String>,
    /// 专业班级（可选）
    #[schema(example = "计算机科学与技术2022级3班")]
    pub class_info: Option<String>,
}

/// 删除用户请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteUserRequest {
    /// 删除原因
    #[schema(example = "违规内容")]
    pub reason: String,
}

/// 批量导入结果
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchImportResult {
    /// 成功导入数量
    #[schema(example = 45)]
    pub imported_count: i32,
    /// 更新数量
    #[schema(example = 5)]
    pub updated_count: i32,
    /// 错误列表
    pub errors: Vec<String>,
}

/// 用户数据库信息（用于管理员查看）
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserDatabaseInfo {
    /// 记录ID
    #[schema(example = 1)]
    pub id: i32,
    /// 身份标识（学号）
    #[schema(example = "2203010301")]
    pub identity_key: String,
    /// 数据库名
    #[schema(example = "db_2203010301")]
    pub db_name: String,
    /// 数据库用户名
    #[schema(example = "user_2203010301")]
    pub db_user: String,
    /// 申请状态
    #[schema(example = "success")]
    pub status: String,
    /// 失败原因（如果有）
    pub failure_reason: Option<String>,
    /// 创建时间
    #[schema(example = "2025-07-14T10:00:00Z")]
    pub created_at: String,
    /// 删除时间（如果已删除）
    pub deleted_at: Option<String>,
    /// 删除原因（如果已删除）
    pub deletion_reason: Option<String>,
}

/// SQLite 数据库中的申请者记录
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Applicant {
    /// 申请记录ID
    #[schema(example = 1)]
    pub id: i32,
    /// 用户身份标识
    #[schema(example = "20250701")]
    pub identity_key: String,
    /// 数据库名称
    #[schema(example = "db_20250701")]
    pub db_name: String,
    /// 数据库用户名
    #[schema(example = "user_20250701")]
    pub db_user: String,
    /// 申请状态 (success, failed, deleted)
    #[schema(example = "success")]
    pub status: String,
    /// 失败原因 (如果申请失败)
    #[schema(example = "")]
    pub failure_reason: Option<String>,
    /// 创建时间
    #[schema(example = "2025-07-13T15:00:00Z")]
    pub created_at: String,
    /// 删除时间 (如果被管理员删除)
    #[schema(example = "")]
    pub deleted_at: Option<String>,
    /// 删除原因 (如果被管理员删除)
    #[schema(example = "")]
    pub deletion_reason: Option<String>,
}

// 业务状态码常量
#[allow(dead_code)]
pub struct StatusCode;

#[allow(dead_code)]
impl StatusCode {
    pub const SUCCESS: i32 = 0;
    pub const INVALID_INPUT: i32 = 40001;
    pub const IDENTITY_EXISTS: i32 = 40901;
    pub const INTERNAL_ERROR: i32 = 50001;
    pub const DB_PROVISION_FAILED: i32 = 50002;
}

// 状态码对应的消息
#[allow(dead_code)]
pub struct StatusMessage;

#[allow(dead_code)]
impl StatusMessage {
    pub const SUCCESS: &'static str = "Success";
    pub const INVALID_INPUT: &'static str = "Invalid input parameter.";
    pub const IDENTITY_EXISTS: &'static str = "Identity key already exists.";
    pub const INTERNAL_ERROR: &'static str = "Internal server error.";
    pub const DB_PROVISION_FAILED: &'static str = "Database provisioning failed.";
}

/// 系统状态信息
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SystemStatus {
    /// 服务启动时间
    pub uptime: String,
    /// 数据库连接状态
    pub database_status: String,
    /// MySQL连接状态
    pub mysql_status: String,
    /// 总申请数量
    pub total_applications: i64,
    /// 今日申请数量
    pub today_applications: i64,
    /// 系统版本
    pub version: String,
}

/// 申请统计信息
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApplicationStats {
    /// 总申请数量
    pub total_count: i64,
    /// 今日申请数量
    pub today_count: i64,
    /// 本周申请数量
    pub week_count: i64,
    /// 本月申请数量
    pub month_count: i64,
    /// 成功申请数量
    pub successful_count: i64,
    /// 失败申请数量
    pub failed_count: i64,
    /// 已删除申请数量
    pub deleted_count: i64,
    /// 最近申请记录
    pub recent_applications: Vec<Applicant>,
}

/// 管理员登录请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminLoginRequest {
    /// 管理员密码
    #[schema(example = "admin_password")]
    pub password: String,
}

/// 管理员删除用户请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminDeleteRequest {
    /// 要删除的身份标识
    #[schema(example = "20250701")]
    pub identity_key: String,
    /// 删除原因
    #[schema(example = "违法内容")]
    pub reason: String,
}

/// 公开申请记录 (不包含敏感信息)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublicApplicationRecord {
    /// 申请记录ID
    #[schema(example = 1)]
    pub id: i32,
    /// 用户身份标识 (脱敏处理)
    #[schema(example = "2025****")]
    pub identity_key_masked: String,
    /// 申请状态
    #[schema(example = "success")]
    pub status: String,
    /// 创建时间
    #[schema(example = "2025-07-13T15:00:00Z")]
    pub created_at: String,
}
