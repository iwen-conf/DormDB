use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// 数据库申请请求
///
/// 用户申请数据库时提交的请求体。
///
/// # 字段说明
/// - `identity_key`: 用户身份标识（编号），必须在白名单中
///
/// # 编号格式要求
/// - 长度：1-50个字符
/// - 字符：字母、数字、下划线、连字符
/// - 限制：必须以字母或数字开头和结尾
///
/// # 示例
/// ```json
/// {
///   "identity_key": "USER123"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApplyRequest {
    /// 用户身份标识（编号）
    ///
    /// 可以是任何合理的用户编号，如：USER123、emp_001、ID-2024-001
    ///
    /// 格式说明：
    /// - 长度：1-50个字符
    /// - 字符：字母、数字、下划线、连字符
    /// - 限制：必须以字母或数字开头和结尾
    #[schema(example = "USER123", min_length = 1, max_length = 50, pattern = r"^[a-zA-Z0-9][a-zA-Z0-9_-]*[a-zA-Z0-9]$")]
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
/// 
/// 申请成功后系统返回的数据库连接信息。
/// 
/// # 安全说明
/// - 密码为系统随机生成的16位强密码
/// - 用户只能访问自己的数据库
/// - 权限仅限于数据操作（SELECT、INSERT、UPDATE、DELETE等）
/// - 禁止结构操作（CREATE、DROP、ALTER等）
/// 
/// # 连接方式
/// 可以使用以下任一方式连接：
/// 1. 使用connection_string（推荐）
/// 2. 使用jdbc_url（Java应用）
/// 3. 使用单独的连接参数
/// 
/// # 注意事项
/// - 请妥善保管密码，系统不会再次提供
/// - 连接字符串包含`allowPublicKeyRetrieval=true`参数，用于MySQL 8.0+兼容性
/// - 数据库名格式：db_学号
/// - 用户名格式：user_学号
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DatabaseCredentials {
    /// 数据库主机地址
    /// 
    /// MySQL服务器的主机地址，通常是localhost或具体的IP地址
    #[schema(example = "localhost")]
    pub db_host: String,
    
    /// 数据库端口
    /// 
    /// MySQL服务器的端口号，默认为3306
    #[schema(example = 3306)]
    pub db_port: u16,
    
    /// 数据库名称
    /// 
    /// 为用户创建的数据库名称，格式为 db_学号
    #[schema(example = "db_2023010101")]
    pub db_name: String,
    
    /// 数据库用户名
    /// 
    /// 为用户创建的数据库用户名，格式为 user_学号
    #[schema(example = "user_2023010101")]
    pub username: String,
    
    /// 数据库密码
    /// 
    /// 系统生成的16位强密码，包含大小写字母、数字和特殊字符
    #[schema(example = "Abc123!@#DefGhi4")]
    pub password: String,
    
    /// 完整的连接字符串 (推荐使用)
    /// 
    /// 包含所有必要参数的MySQL连接字符串，可直接用于大多数数据库客户端
    #[schema(
        example = "mysql://user_2023010101:Abc123!@#DefGhi4@localhost:3306/db_2023010101?allowPublicKeyRetrieval=true&useSSL=false"
    )]
    pub connection_string: String,
    
    /// JDBC 连接字符串 (Java应用使用)
    /// 
    /// 专门为Java应用程序设计的JDBC连接字符串
    #[schema(
        example = "jdbc:mysql://localhost:3306/db_2023010101?allowPublicKeyRetrieval=true&useSSL=false&user=user_2023010101&password=Abc123!@#DefGhi4"
    )]
    pub jdbc_url: String,
}

/// 用户编号管理记录
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct StudentId {
    /// 记录ID
    #[schema(example = 1)]
    pub id: i32,
    /// 用户编号
    #[schema(example = "USER123")]
    pub student_id: String,
    /// 用户姓名（可选）
    #[schema(example = "张三")]
    pub student_name: Option<String>,
    /// 班级信息（可选）
    #[schema(example = "计算机科学与技术2022级3班")]
    pub class_info: Option<String>,
    /// 是否已申请数据库
    #[schema(example = false)]
    pub has_applied: bool,
    /// 申请的数据库名（如果已申请）
    #[schema(example = "db_USER123")]
    pub applied_db_name: Option<String>,
    /// 创建时间
    #[schema(example = "2025-07-14T10:00:00Z")]
    pub created_at: String,
    /// 更新时间
    #[schema(example = "2025-07-14T10:00:00Z")]
    pub updated_at: String,
}

/// 用户编号批量导入请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StudentIdBatchImport {
    /// 用户编号列表，每行一个编号，格式：编号,姓名,班级（姓名和班级可选）
    #[schema(example = "USER123,张三,计算机2022-3班\nUSER124,李四,计算机2022-3班")]
    pub student_data: String,
    /// 是否覆盖已存在的编号
    #[schema(example = false)]
    pub overwrite_existing: bool,
}

/// 用户编号管理统计
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StudentIdStats {
    /// 总编号数
    #[schema(example = 150)]
    pub total_count: i32,
    /// 已申请数据库的编号数
    #[schema(example = 45)]
    pub applied_count: i32,
    /// 未申请数据库的编号数
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

/// 添加用户编号请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AddStudentIdRequest {
    /// 用户编号
    #[schema(example = "USER123")]
    pub student_id: String,
    /// 用户姓名（可选）
    #[schema(example = "张三")]
    pub student_name: Option<String>,
    /// 班级信息（可选）
    #[schema(example = "计算机科学与技术2022级3班")]
    pub class_info: Option<String>,
}

/// 更新用户编号请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateStudentIdRequest {
    /// 用户姓名（可选）
    #[schema(example = "张三")]
    pub student_name: Option<String>,
    /// 班级信息（可选）
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
    #[schema(example = "运行中")]
    pub uptime: String,
    /// 数据库连接状态
    #[schema(example = "正常")]
    pub database_status: String,
    /// MySQL连接状态
    #[schema(example = "正常")]
    pub mysql_status: String,
    /// 总申请数量
    #[schema(example = 156)]
    pub total_applications: i64,
    /// 今日申请数量
    #[schema(example = 12)]
    pub today_applications: i64,
    /// 系统版本
    #[schema(example = "1.0.0")]
    pub version: String,
}

/// 申请统计信息
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApplicationStats {
    /// 总申请数量
    #[schema(example = 156)]
    pub total_count: i64,
    /// 今日申请数量
    #[schema(example = 12)]
    pub today_count: i64,
    /// 本周申请数量
    #[schema(example = 45)]
    pub week_count: i64,
    /// 本月申请数量
    #[schema(example = 89)]
    pub month_count: i64,
    /// 成功申请数量
    #[schema(example = 142)]
    pub successful_count: i64,
    /// 失败申请数量
    #[schema(example = 8)]
    pub failed_count: i64,
    /// 已删除申请数量
    #[schema(example = 6)]
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
