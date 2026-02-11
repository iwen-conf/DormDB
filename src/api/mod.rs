use crate::models::{
    AddStudentIdRequest, AdminDeleteRequest, AdminLoginRequest, ApiResponse, Applicant,
    ApplicationStats, ApplyRequest, BatchImportResult, DatabaseCredentials, DeleteUserRequest,
    PaginationQuery, PublicApplicationRecord, StudentId, StudentIdBatchImport, StudentIdStats,
    SystemStatus, UpdateStudentIdRequest, UserDatabaseInfo,
};
use crate::services::DatabaseService;
use actix_web::{HttpResponse, Result, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use log::{info, warn};
use utoipa::OpenApi;

/// 申请数据库
///
/// 为用户申请一个新的MySQL数据库实例，包括创建数据库、用户和授权。
///
/// # 安全特性
/// - 灵活的用户编号格式验证
/// - 用户编号白名单验证
/// - 自动生成安全密码
/// - 权限最小化原则
///
/// # 申请流程
/// 1. 验证用户编号格式和白名单
/// 2. 生成安全的数据库名和用户名
/// 3. 创建MySQL数据库和用户
/// 4. 授予最小必要权限
/// 5. 记录申请信息到SQLite
///
/// # 权限说明
/// 用户获得的权限仅限于：
/// - SELECT, INSERT, UPDATE, DELETE (数据操作)
/// - INDEX, LOCK TABLES (索引和锁定)
///
/// 禁止权限：
/// - CREATE, DROP, ALTER (结构修改)
/// - GRANT, SUPER (权限管理)
///
/// # 错误处理
/// - 40001: 用户编号格式无效
/// - 40901: 用户编号已申请过数据库
/// - 50002: 数据库创建失败
#[utoipa::path(
    post,
    path = "/api/v1/apply",
    tag = "数据库申请",
    operation_id = "apply_database",
    request_body(
        content = ApplyRequest,
        description = "数据库申请请求",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "申请成功", body = ApiResponse<DatabaseCredentials>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": {
                 "db_host": "localhost",
                 "db_port": 3306,
                 "db_name": "db_2023010101",
                 "username": "user_2023010101",
                 "password": "Abc123!@#DefGhi4",
                 "connection_string": "mysql://user_2023010101:Abc123!@#DefGhi4@localhost:3306/db_2023010101?allowPublicKeyRetrieval=true&useSSL=false",
                 "jdbc_url": "jdbc:mysql://localhost:3306/db_2023010101?allowPublicKeyRetrieval=true&useSSL=false&user=user_2023010101&password=Abc123!@#DefGhi4"
             }
         })),
        (status = 400, description = "请求参数无效", body = ApiResponse<String>,
         example = json!({
             "code": 40001,
             "message": "Invalid input parameter.",
             "data": null
         })),
        (status = 409, description = "用户编号已存在", body = ApiResponse<String>,
         example = json!({
             "code": 40901,
             "message": "Identity key already exists.",
             "data": null
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50002,
             "message": "Database provisioning failed.",
             "data": null
         }))
    ),
    security(
        // 此接口无需认证
    )
)]
pub async fn apply_database(
    request: web::Json<ApplyRequest>,
    service: web::Data<DatabaseService>,
) -> Result<HttpResponse> {
    info!("收到用户身份标识的申请请求: {}", request.identity_key);

    let response = service.apply_database(&request.identity_key).await;

    let http_status = match response.code {
        0 => 200,
        40001 => 400,
        40901 => 409,
        50001 | 50002 => 500,
        _ => 500,
    };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 获取所有申请者信息
///
/// 管理员接口，用于查看所有数据库申请记录。
///
/// # 功能说明
/// - 返回所有申请记录，包括成功、失败和已删除的记录
/// - 按创建时间倒序排列
/// - 包含申请状态、失败原因、删除信息等详细数据
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    get,
    path = "/api/v1/applicants",
    tag = "管理员功能",
    operation_id = "get_all_applicants",
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<Vec<Applicant>>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": [
                 {
                     "id": 1,
                     "identity_key": "2023010101",
                     "db_name": "db_2023010101",
                     "db_user": "user_2023010101",
                     "status": "success",
                     "failure_reason": null,
                     "created_at": "2025-07-15T10:00:00Z",
                     "deleted_at": null,
                     "deletion_reason": null
                 },
                 {
                     "id": 2,
                     "identity_key": "2023010102",
                     "db_name": "db_2023010102",
                     "db_user": "user_2023010102",
                     "status": "deleted",
                     "failure_reason": null,
                     "created_at": "2025-07-14T09:00:00Z",
                     "deleted_at": "2025-07-15T11:00:00Z",
                     "deletion_reason": "违规内容"
                 }
             ]
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "Internal server error.",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn get_applicants(service: web::Data<DatabaseService>) -> Result<HttpResponse> {
    info!("收到获取所有申请者信息的请求");

    let response = service.get_all_applicants().await;

    let http_status = match response.code {
        0 => 200,
        _ => 500,
    };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 健康检查
///
/// 检查服务是否正常运行。
///
/// # 功能说明
/// - 快速检查服务状态
/// - 无需认证即可访问
/// - 用于负载均衡器和监控系统的健康检查
///
/// # 返回信息
/// 始终返回成功状态，表示服务正在运行
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "公开接口",
    operation_id = "health_check",
    responses(
        (status = 200, description = "服务正常运行", body = ApiResponse<String>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": "服务运行正常"
         }))
    ),
    security(
        // 此接口无需认证
    )
)]
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(ApiResponse::success("服务运行正常".to_string())))
}

/// 获取系统状态
///
/// 管理员接口，获取详细的系统运行状态。
///
/// # 功能说明
/// - 检查SQLite和MySQL数据库连接状态
/// - 获取申请统计数据
/// - 显示系统版本和运行时间
/// - 用于系统监控和故障诊断
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    get,
    path = "/api/v1/admin/status",
    tag = "管理员功能",
    operation_id = "get_system_status",
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<SystemStatus>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": {
                 "uptime": "运行中",
                 "database_status": "正常",
                 "mysql_status": "正常",
                 "total_applications": 156,
                 "today_applications": 12,
                 "version": "1.0.0"
             }
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "Internal server error.",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn get_system_status(service: web::Data<DatabaseService>) -> Result<HttpResponse> {
    info!("管理员请求系统状态");

    let response = service.get_system_status().await;

    let http_status = match response.code {
        0 => 200,
        _ => 500,
    };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 获取申请统计
///
/// 管理员接口，获取申请记录的统计信息。
///
/// # 功能说明
/// - 提供详细的申请数据统计
/// - 包含时间维度统计（今日、本周、本月）
/// - 按状态分类统计（成功、失败、已删除）
/// - 返回最近的申请记录
/// - 用于数据分析和报表生成
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    get,
    path = "/api/v1/admin/stats",
    tag = "管理员功能",
    operation_id = "get_application_stats",
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<ApplicationStats>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": {
                 "total_count": 156,
                 "today_count": 12,
                 "week_count": 45,
                 "month_count": 89,
                 "successful_count": 142,
                 "failed_count": 8,
                 "deleted_count": 6,
                 "recent_applications": [
                     {
                         "id": 1,
                         "identity_key": "2023010101",
                         "db_name": "db_2023010101",
                         "db_user": "user_2023010101",
                         "status": "success",
                         "failure_reason": null,
                         "created_at": "2025-07-15T10:00:00Z",
                         "deleted_at": null,
                         "deletion_reason": null
                     }
                 ]
             }
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "Internal server error.",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn get_application_stats(service: web::Data<DatabaseService>) -> Result<HttpResponse> {
    info!("管理员请求申请统计");

    let response = service.get_application_stats().await;

    let http_status = match response.code {
        0 => 200,
        _ => 500,
    };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 检查和修复数据一致性
///
/// 管理员接口，检查MySQL和SQLite之间的数据一致性并自动修复。
///
/// # 功能说明
/// - 检查SQLite记录与MySQL实际数据库的一致性
/// - 自动修复发现的不一致问题
/// - 清理孤立的数据库或记录
/// - 生成详细的修复报告
/// - 用于系统维护和数据完整性保障
///
/// # 修复内容
/// - 删除SQLite中存在但MySQL中不存在的记录
/// - 清理MySQL中存在但SQLite中无记录的数据库
/// - 更新状态不一致的记录
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    post,
    path = "/api/v1/admin/repair",
    tag = "管理员功能",
    operation_id = "check_and_repair_consistency",
    responses(
        (status = 200, description = "检查和修复完成", body = ApiResponse<String>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": "数据一致性检查完成，修复了3个不一致项"
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "Internal server error.",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn check_and_repair_consistency(
    service: web::Data<DatabaseService>,
) -> Result<HttpResponse> {
    info!("管理员请求数据一致性检查和修复");

    let response = service.check_and_repair_consistency().await;

    let http_status = match response.code {
        0 => 200,
        _ => 500,
    };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 管理员登录验证
///
/// 验证管理员密码并返回JWT访问令牌。
///
/// # 安全特性
/// - 密码强度验证（至少8位，包含大小写字母、数字、特殊字符）
/// - bcrypt哈希验证
/// - JWT令牌生成（24小时有效期）
/// - 会话管理和追踪
///
/// # 认证流程
/// 1. 验证密码格式和强度
/// 2. 使用bcrypt验证密码哈希
/// 3. 生成JWT令牌（包含用户信息和权限）
/// 4. 返回令牌用于后续API调用
///
/// # 使用方法
/// 1. 调用此接口获取JWT令牌
/// 2. 在后续管理员API请求中包含令牌：
///    ```text
///    Authorization: Bearer YOUR_JWT_TOKEN
///    ```
///
/// # 密码要求
/// - 最少8位字符
/// - 包含大写字母 (A-Z)
/// - 包含小写字母 (a-z)
/// - 包含数字 (0-9)
/// - 包含特殊字符 (!@#$%^&*)
///
/// # 错误处理
/// - 40101: 密码错误
/// - 40102: 密码强度不足
/// - 50003: 令牌生成失败
/// - 50004: 认证服务异常
#[utoipa::path(
    post,
    path = "/api/v1/admin/login",
    tag = "管理员功能",
    operation_id = "admin_login",
    request_body(
        content = AdminLoginRequest,
        description = "管理员登录请求",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "登录成功", body = ApiResponse<String>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": "{\"token\": \"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbiIsImV4cCI6MTcyMTI5NDQwMCwiaWF0IjoxNzIxMjA4MDAwLCJyb2xlIjoiYWRtaW4iLCJzZXNzaW9uX2lkIjoiNTU1ZTU1ZTUtNTU1ZS01NTVlLTU1NWUtNTU1ZTU1ZTU1ZTU1In0.example_signature\", \"message\": \"登录成功\"}"
         })),
        (status = 401, description = "认证失败", body = ApiResponse<String>,
         example = json!({
             "code": 40101,
             "message": "密码错误",
             "data": null
         })),
        (status = 400, description = "密码强度不足", body = ApiResponse<String>,
         example = json!({
             "code": 40102,
             "message": "密码强度不足",
             "data": null
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50003,
             "message": "令牌生成失败",
             "data": null
         }))
    ),
    security(
        // 此接口用于获取认证令牌，本身无需认证
    )
)]
pub async fn admin_login(
    service: web::Data<DatabaseService>,
    request: web::Json<AdminLoginRequest>,
) -> Result<HttpResponse> {
    info!("管理员登录尝试");

    let response = service.admin_login(&request.password).await;

    let http_status = match response.code {
        0 => 200,
        40101 => 401,
        40102 => 400,
        50003 | 50004 => 500,
        _ => 500,
    };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 管理员删除用户
///
/// 删除用户的数据库和用户账号。
///
/// # 功能说明
/// - 删除用户在MySQL中的数据库和用户账号
/// - 在SQLite中标记记录为已删除状态
/// - 记录删除原因和时间
/// - 支持内容审核和违规处理
///
/// # 删除流程
/// 1. 验证用户存在性
/// 2. 删除MySQL数据库和用户
/// 3. 更新SQLite记录状态
/// 4. 记录删除原因和时间戳
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    post,
    path = "/api/v1/admin/delete",
    tag = "管理员功能",
    operation_id = "admin_delete_user",
    request_body(
        content = AdminDeleteRequest,
        description = "删除用户请求",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "删除成功", body = ApiResponse<String>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": "用户 2023010101 已被删除"
         })),
        (status = 400, description = "请求参数无效", body = ApiResponse<String>,
         example = json!({
             "code": 40001,
             "message": "用户不存在或已被删除",
             "data": null
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "删除用户失败: 数据库操作异常",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn admin_delete_user(
    service: web::Data<DatabaseService>,
    request: web::Json<AdminDeleteRequest>,
) -> Result<HttpResponse> {
    warn!(
        "管理员删除用户: {}, 原因: {}",
        request.identity_key, request.reason
    );

    let response = service
        .admin_delete_user(&request.identity_key, &request.reason)
        .await;

    let http_status = match response.code {
        0 => 200,
        _ => 500,
    };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 获取公开申请记录
///
/// 获取脱敏处理后的申请记录，供公开展示。
///
/// # 功能说明
/// - 返回脱敏处理后的申请记录
/// - 隐藏敏感信息（如完整学号）
/// - 仅显示申请状态和时间
/// - 用于公开统计和展示
/// - 无需认证即可访问
///
/// # 数据脱敏
/// - 学号显示为 "2023****" 格式
/// - 不包含数据库连接信息
/// - 不包含失败原因等敏感信息
#[utoipa::path(
    get,
    path = "/api/v1/public/applications",
    tag = "公开接口",
    operation_id = "get_public_applications",
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<Vec<PublicApplicationRecord>>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": [
                 {
                     "id": 1,
                     "identity_key_masked": "2023****",
                     "status": "success",
                     "created_at": "2025-07-15T10:00:00Z"
                 },
                 {
                     "id": 2,
                     "identity_key_masked": "2023****",
                     "status": "success",
                     "created_at": "2025-07-15T09:30:00Z"
                 }
             ]
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "Internal server error.",
             "data": null
         }))
    ),
    security(
        // 此接口无需认证
    )
)]
pub async fn get_public_applications(service: web::Data<DatabaseService>) -> Result<HttpResponse> {
    info!("获取公开申请记录");

    let response = service.get_public_applications().await;

    let http_status = match response.code {
        0 => 200,
        _ => 500,
    };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

// 用户编号管理 API

/// 获取用户编号列表
///
/// 获取系统中所有用户编号记录，支持分页查询。
///
/// # 功能说明
/// - 支持分页查询，避免大量数据传输
/// - 显示用户编号的申请状态
/// - 包含用户姓名和班级信息
/// - 按创建时间倒序排列
///
/// # 分页参数
/// - `limit`: 每页返回的记录数，默认100，最大500
/// - `offset`: 跳过的记录数，默认0
///
/// # 返回字段说明
/// - `id`: 记录唯一标识
/// - `student_id`: 用户编号
/// - `student_name`: 用户姓名（可选）
/// - `class_info`: 班级信息（可选）
/// - `has_applied`: 是否已申请数据库
/// - `applied_db_name`: 申请的数据库名（如果已申请）
/// - `created_at`: 创建时间
/// - `updated_at`: 更新时间
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    get,
    path = "/api/v1/admin/student-ids",
    tag = "用户编号管理",
    operation_id = "get_student_ids",
    params(
        ("limit" = Option<i32>, Query, description = "每页数量，默认100，最大500"),
        ("offset" = Option<i32>, Query, description = "偏移量，默认0")
    ),
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<Vec<StudentId>>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": [
                 {
                     "id": 1,
                     "student_id": "USER123",
                     "student_name": "张三",
                     "class_info": "计算机科学与技术2023级1班",
                     "has_applied": false,
                     "applied_db_name": null,
                     "created_at": "2025-07-15T10:00:00Z",
                     "updated_at": "2025-07-15T10:00:00Z"
                 },
                 {
                     "id": 2,
                     "student_id": "EMP001",
                     "student_name": "李四",
                     "class_info": "计算机科学与技术2023级1班",
                     "has_applied": true,
                     "applied_db_name": "db_EMP001",
                     "created_at": "2025-07-15T09:00:00Z",
                     "updated_at": "2025-07-15T11:00:00Z"
                 }
             ]
         })),
        (status = 401, description = "未授权访问", body = ApiResponse<String>,
         example = json!({
             "code": 40101,
             "message": "Unauthorized",
             "data": null
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "Internal server error.",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn api_get_student_ids(
    data: web::Data<DatabaseService>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse> {
    info!("管理员请求学号列表");

    let response = data.get_student_ids(query.limit, query.offset).await;
    let http_status = if response.code == 0 { 200 } else { 500 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 添加用户编号
///
/// 管理员接口，添加单个用户编号到白名单。
///
/// # 功能说明
/// - 添加学号到系统白名单
/// - 支持添加学生姓名和班级信息
/// - 验证学号格式的有效性
/// - 防止重复添加
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    post,
    path = "/api/v1/admin/student-ids",
    tag = "用户编号管理",
    operation_id = "add_student_id",
    request_body(
        content = AddStudentIdRequest,
        description = "添加学号请求",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "添加成功", body = ApiResponse<String>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": "学号添加成功"
         })),
        (status = 400, description = "请求参数无效或学号已存在", body = ApiResponse<String>,
         example = json!({
             "code": 40001,
             "message": "学号已存在",
             "data": null
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "添加学号失败",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn api_add_student_id(
    data: web::Data<DatabaseService>,
    req: web::Json<AddStudentIdRequest>,
) -> Result<HttpResponse> {
    info!("管理员添加学号: {}", req.student_id);

    let response = data
        .add_student_id(
            &req.student_id,
            req.student_name.as_deref(),
            req.class_info.as_deref(),
        )
        .await;
    let http_status = if response.code == 0 { 200 } else { 400 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 批量导入学号
///
/// 管理员接口，批量导入学号到白名单。
///
/// # 功能说明
/// - 支持批量导入大量学号
/// - 支持CSV格式数据（学号,姓名,班级）
/// - 可选择是否覆盖已存在的学号
/// - 返回详细的导入结果统计
/// - 提供错误信息用于问题排查
///
/// # 数据格式
/// 每行一个学号记录，格式：学号,姓名,班级
/// 姓名和班级为可选字段
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    post,
    path = "/api/v1/admin/student-ids/batch-import",
    tag = "学号管理",
    operation_id = "batch_import_student_ids",
    request_body(
        content = StudentIdBatchImport,
        description = "批量导入学号请求",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "导入完成", body = ApiResponse<BatchImportResult>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": {
                 "imported_count": 45,
                 "updated_count": 5,
                 "errors": [
                     "第3行：学号格式无效",
                     "第7行：学号已存在且未选择覆盖"
                 ]
             }
         })),
        (status = 400, description = "请求参数无效", body = ApiResponse<String>,
         example = json!({
             "code": 40001,
             "message": "导入数据格式错误",
             "data": null
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "批量导入失败",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn api_batch_import_student_ids(
    data: web::Data<DatabaseService>,
    req: web::Json<StudentIdBatchImport>,
) -> Result<HttpResponse> {
    info!("管理员批量导入学号");

    let response = data
        .batch_import_student_ids(&req.student_data, req.overwrite_existing)
        .await;
    let http_status = if response.code == 0 { 200 } else { 400 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 更新学号信息
///
/// 管理员接口，更新指定学号的信息。
///
/// # 功能说明
/// - 更新学号对应的学生姓名和班级信息
/// - 不能修改学号本身
/// - 支持部分字段更新
/// - 验证记录存在性
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    put,
    path = "/api/v1/admin/student-ids/{id}",
    tag = "学号管理",
    operation_id = "update_student_id",
    params(
        ("id" = i32, Path, description = "学号记录ID")
    ),
    request_body(
        content = UpdateStudentIdRequest,
        description = "更新学号信息请求",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "更新成功", body = ApiResponse<String>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": "学号信息更新成功"
         })),
        (status = 400, description = "请求参数无效或记录不存在", body = ApiResponse<String>,
         example = json!({
             "code": 40001,
             "message": "学号记录不存在",
             "data": null
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "更新学号信息失败",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn api_update_student_id(
    data: web::Data<DatabaseService>,
    path: web::Path<i32>,
    req: web::Json<UpdateStudentIdRequest>,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    info!("管理员更新学号信息: ID {}", id);

    let response = data
        .update_student_id(id, req.student_name.as_deref(), req.class_info.as_deref())
        .await;
    let http_status = if response.code == 0 { 200 } else { 400 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 删除学号
///
/// 管理员接口，从白名单中删除指定学号。
///
/// # 功能说明
/// - 从系统白名单中删除学号
/// - 验证学号是否已申请数据库
/// - 如果已申请数据库，需要先删除用户数据库
/// - 永久删除记录，无法恢复
///
/// # 注意事项
/// 删除已申请数据库的学号前，请先删除对应的用户数据库
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    delete,
    path = "/api/v1/admin/student-ids/{id}",
    tag = "学号管理",
    operation_id = "delete_student_id",
    params(
        ("id" = i32, Path, description = "学号记录ID")
    ),
    responses(
        (status = 200, description = "删除成功", body = ApiResponse<String>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": "学号删除成功"
         })),
        (status = 400, description = "请求参数无效或记录不存在", body = ApiResponse<String>,
         example = json!({
             "code": 40001,
             "message": "学号记录不存在或已申请数据库",
             "data": null
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "删除学号失败",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn api_delete_student_id(
    data: web::Data<DatabaseService>,
    path: web::Path<i32>,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    info!("管理员删除学号: ID {}", id);

    let response = data.delete_student_id(id).await;
    let http_status = if response.code == 0 { 200 } else { 400 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 获取学号统计
///
/// 管理员接口，获取学号白名单的统计信息。
///
/// # 功能说明
/// - 统计白名单中的学号总数
/// - 统计已申请数据库的学号数量
/// - 统计未申请数据库的学号数量
/// - 用于数据分析和容量规划
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    get,
    path = "/api/v1/admin/student-ids/stats",
    tag = "学号管理",
    operation_id = "get_student_id_stats",
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<StudentIdStats>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": {
                 "total_count": 150,
                 "applied_count": 45,
                 "not_applied_count": 105
             }
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "获取学号统计失败",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn api_get_student_id_stats(data: web::Data<DatabaseService>) -> Result<HttpResponse> {
    info!("管理员请求学号统计");

    let response = data.get_student_id_stats().await;
    let http_status = if response.code == 0 { 200 } else { 500 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// DormDB API 文档
///
/// 数据库自助申请平台API接口文档
///
/// # 认证说明
/// 管理员接口需要JWT Bearer Token认证：
/// ```text
/// Authorization: Bearer YOUR_JWT_TOKEN
/// ```
///
/// # 获取令牌
/// 调用 `/api/v1/admin/login` 接口获取JWT令牌
///
/// # 安全特性
/// - JWT令牌有效期24小时
/// - 密码采用bcrypt哈希存储
/// - 严格的输入验证和SQL注入防护
/// - 用户编号白名单验证机制
///
/// # 版本信息
/// - 版本: 1.0.0
/// - 协议: MIT License
/// - 技术栈: Rust + Actix-web + MySQL + SQLite
#[derive(OpenApi)]
#[openapi(
    info(
        title = "DormDB API",
        version = "1.0.0",
        description = "数据库自助申请平台API接口文档\n\n## 功能特性\n- 🔐 用户编号白名单验证\n- 🚀 自动数据库创建\n- 🛡️ JWT安全认证\n- 📊 完整的管理面板\n- 🔧 数据一致性检查\n\n## 使用流程\n1. 管理员导入用户编号白名单\n2. 用户使用编号申请数据库\n3. 系统自动创建MySQL数据库和用户\n4. 返回连接信息供用户使用\n\n## 安全保障\n- 严格的权限控制（只能访问自己的数据库）\n- 输入验证和SQL注入防护\n- 密码强度验证和bcrypt哈希\n- JWT令牌认证和会话管理",
        contact(
            name = "DormDB Team",
            email = "admin@dormdb.com",
            url = "https://github.com/iwen-conf/DormDB"
        ),
        license(
            name = "MIT License",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "开发环境"),
        (url = "https://your-domain.com", description = "生产环境")
    ),
    paths(
        apply_database,
        get_applicants,
        health_check,
        get_system_status,
        get_application_stats,
        check_and_repair_consistency,
        admin_login,
        admin_delete_user,
        get_public_applications,
        api_get_student_ids,
        api_add_student_id,
        api_batch_import_student_ids,
        api_update_student_id,
        api_delete_student_id,
        api_get_student_id_stats,
        api_get_all_users,
        api_delete_user_by_identity
    ),
    components(
        schemas(
            ApplyRequest,
            DatabaseCredentials,
            Applicant,
            SystemStatus,
            ApplicationStats,
            AdminLoginRequest,
            AdminDeleteRequest,
            PublicApplicationRecord,
            StudentId,
            StudentIdBatchImport,
            StudentIdStats,
            PaginationQuery,
            AddStudentIdRequest,
            UpdateStudentIdRequest,
            BatchImportResult,
            UserDatabaseInfo,
            DeleteUserRequest,
            ApiResponse<DatabaseCredentials>,
            ApiResponse<Vec<UserDatabaseInfo>>,
            ApiResponse<Vec<Applicant>>,
            ApiResponse<Vec<StudentId>>,
            ApiResponse<StudentIdStats>,
            ApiResponse<BatchImportResult>,
            ApiResponse<String>,
            ApiResponse<SystemStatus>,
            ApiResponse<ApplicationStats>,
            ApiResponse<Vec<PublicApplicationRecord>>
        )
    ),
    tags(
        (name = "数据库申请", description = "用户数据库申请相关接口\n\n用户使用编号申请MySQL数据库实例，系统自动创建数据库、用户并分配权限。"),
        (name = "管理员功能", description = "管理员认证和系统管理接口\n\n包括管理员登录、系统状态监控、数据一致性检查等功能。"),
        (name = "用户编号管理", description = "用户编号白名单管理接口\n\n管理员可以添加、删除、批量导入用户编号，只有白名单中的编号才能申请数据库。"),
        (name = "用户管理", description = "用户数据库管理接口\n\n管理员可以查看所有用户及其数据库，删除用户及其数据库实例。"),
        (name = "公开接口", description = "无需认证的公开接口\n\n包括健康检查、公开申请记录等功能。"),
        (name = "系统监控", description = "系统状态和统计信息接口\n\n提供系统运行状态、申请统计、性能指标等信息。")
    ),
    external_docs(
        url = "https://github.com/iwen-conf/DormDB/blob/main/用户手册.md",
        description = "查看完整的用户手册和部署指南"
    )
)]
pub struct ApiDoc;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/apply", web::post().to(apply_database))
            .route("/health", web::get().to(health_check))
            .route("/admin/login", web::post().to(admin_login))
            // 公开接口
            .route(
                "/public/applications",
                web::get().to(get_public_applications),
            )
            // 需要管理员权限的接口
            .service(
                web::resource("/applicants")
                    .wrap(HttpAuthentication::bearer(
                        crate::auth::admin_auth_middleware,
                    ))
                    .route(web::get().to(get_applicants)),
            )
            .service(
                web::scope("/admin")
                    .wrap(HttpAuthentication::bearer(
                        crate::auth::admin_auth_middleware,
                    ))
                    .route("/status", web::get().to(get_system_status))
                    .route("/stats", web::get().to(get_application_stats))
                    .route("/repair", web::post().to(check_and_repair_consistency))
                    .route("/delete", web::post().to(admin_delete_user))
                    .route("/student-ids", web::get().to(api_get_student_ids))
                    .route("/student-ids", web::post().to(api_add_student_id))
                    .route(
                        "/student-ids/batch-import",
                        web::post().to(api_batch_import_student_ids),
                    )
                    .route("/student-ids/{id}", web::put().to(api_update_student_id))
                    .route("/student-ids/{id}", web::delete().to(api_delete_student_id))
                    .route(
                        "/student-ids/stats",
                        web::get().to(api_get_student_id_stats),
                    )
                    .route("/users", web::get().to(api_get_all_users))
                    .route(
                        "/users/{identity_key}",
                        web::delete().to(api_delete_user_by_identity),
                    ),
            ),
    );
}

// 用户管理 API

/// 获取所有用户列表
///
/// 管理员接口，获取所有已申请数据库的用户信息。
///
/// # 功能说明
/// - 返回所有成功申请数据库的用户记录
/// - 包含用户身份标识、数据库信息、申请状态等
/// - 按创建时间倒序排列
/// - 用于用户管理和监控
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    tag = "用户管理",
    operation_id = "get_all_users",
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<Vec<UserDatabaseInfo>>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": [
                 {
                     "id": 1,
                     "identity_key": "2023010101",
                     "db_name": "db_2023010101",
                     "db_user": "user_2023010101",
                     "status": "success",
                     "failure_reason": null,
                     "created_at": "2025-07-15T10:00:00Z",
                     "deleted_at": null,
                     "deletion_reason": null
                 }
             ]
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "获取用户列表失败",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn api_get_all_users(data: web::Data<DatabaseService>) -> Result<HttpResponse> {
    info!("管理员请求用户列表");

    let response = data.get_all_users().await;
    let http_status = if response.code == 0 { 200 } else { 500 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 删除用户
///
/// 管理员接口，通过身份标识删除用户及其数据库。
///
/// # 功能说明
/// - 通过身份标识（学号）删除用户
/// - 删除用户在MySQL中的数据库和账号
/// - 在SQLite中标记为已删除状态
/// - 记录删除原因和时间戳
/// - 用于内容审核和违规处理
///
/// # 删除流程
/// 1. 验证用户存在性
/// 2. 删除MySQL数据库和用户账号
/// 3. 更新SQLite记录状态
/// 4. 记录删除原因
///
/// # 权限要求
/// 需要管理员JWT令牌
#[utoipa::path(
    delete,
    path = "/api/v1/admin/users/{identity_key}",
    tag = "用户管理",
    operation_id = "delete_user_by_identity",
    params(
        ("identity_key" = String, Path, description = "用户身份标识（学号）")
    ),
    request_body(
        content = DeleteUserRequest,
        description = "删除用户请求",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "删除成功", body = ApiResponse<String>,
         example = json!({
             "code": 0,
             "message": "Success",
             "data": "用户删除成功"
         })),
        (status = 400, description = "请求参数无效或用户不存在", body = ApiResponse<String>,
         example = json!({
             "code": 40001,
             "message": "用户不存在或已被删除",
             "data": null
         })),
        (status = 500, description = "服务器内部错误", body = ApiResponse<String>,
         example = json!({
             "code": 50001,
             "message": "删除用户失败",
             "data": null
         }))
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn api_delete_user_by_identity(
    data: web::Data<DatabaseService>,
    path: web::Path<String>,
    req: web::Json<DeleteUserRequest>,
) -> Result<HttpResponse> {
    let identity_key = path.into_inner();
    info!("管理员删除用户: {}", identity_key);

    let response = data
        .delete_user_by_identity(&identity_key, &req.reason)
        .await;
    let http_status = if response.code == 0 { 200 } else { 400 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}
