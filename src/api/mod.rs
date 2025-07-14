use crate::models::{
    AdminDeleteRequest, AdminLoginRequest, ApiResponse, Applicant, ApplicationStats, ApplyRequest,
    DatabaseCredentials, PublicApplicationRecord, SystemStatus, StudentId, StudentIdBatchImport,
    StudentIdStats, PaginationQuery, AddStudentIdRequest, UpdateStudentIdRequest, BatchImportResult,
    UserDatabaseInfo, DeleteUserRequest,
};
use crate::services::DatabaseService;
use actix_web::{HttpResponse, Result, web};
use log::{info, warn};
use utoipa::OpenApi;

/// 申请数据库
///
/// 为用户申请一个新的数据库实例，包括创建数据库、用户和授权
#[utoipa::path(
    post,
    path = "/api/v1/apply",
    tag = "数据库申请",
    operation_id = "apply_database",
    request_body = ApplyRequest,
    responses(
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
/// 管理员接口，用于查看所有数据库申请记录
#[utoipa::path(
    get,
    path = "/api/v1/applicants",
    tag = "管理员功能",
    operation_id = "get_all_applicants",
    responses(
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
/// 检查服务是否正常运行
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "系统",
    operation_id = "health_check",
    responses(
    )
)]
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(ApiResponse::success("服务运行正常".to_string())))
}

/// 获取系统状态
///
/// 管理员接口，获取详细的系统运行状态
#[utoipa::path(
    get,
    path = "/api/v1/admin/status",
    tag = "管理员功能",
    operation_id = "get_system_status",
    responses(
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
/// 管理员接口，获取申请记录的统计信息
#[utoipa::path(
    get,
    path = "/api/v1/admin/stats",
    tag = "管理员功能",
    operation_id = "get_application_stats",
    responses(
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
/// 管理员接口，检查MySQL和SQLite之间的数据一致性并自动修复
#[utoipa::path(
    post,
    path = "/api/v1/admin/repair",
    tag = "管理员功能",
    operation_id = "check_and_repair_consistency",
    responses(
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
/// 验证管理员密码
#[utoipa::path(
    post,
    path = "/api/v1/admin/login",
    tag = "管理员功能",
    operation_id = "admin_login",
    request_body = AdminLoginRequest,
    responses(
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
        _ => 401,
    };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 管理员删除用户
///
/// 删除用户的数据库和用户账号
#[utoipa::path(
    post,
    path = "/api/v1/admin/delete",
    tag = "管理员功能",
    operation_id = "admin_delete_user",
    request_body = AdminDeleteRequest,
    responses(
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
/// 获取脱敏处理后的申请记录，供公开展示
#[utoipa::path(
    get,
    path = "/api/v1/public/applications",
    tag = "公开接口",
    operation_id = "get_public_applications",
    responses(
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

// 学号管理 API

/// 获取学号列表
#[utoipa::path(
    get,
    path = "/api/v1/admin/student-ids",
    tag = "Student Management",
    params(
    ),
    responses(
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

/// 添加学号
#[utoipa::path(
    post,
    path = "/api/v1/admin/student-ids",
    tag = "Student Management",
    request_body = AddStudentIdRequest,
    responses(
    )
)]
pub async fn api_add_student_id(
    data: web::Data<DatabaseService>,
    req: web::Json<AddStudentIdRequest>,
) -> Result<HttpResponse> {
    info!("管理员添加学号: {}", req.student_id);

    let response = data.add_student_id(&req.student_id, req.student_name.as_deref(), req.class_info.as_deref()).await;
    let http_status = if response.code == 0 { 200 } else { 400 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 批量导入学号
#[utoipa::path(
    post,
    path = "/api/v1/admin/student-ids/batch-import",
    tag = "Student Management",
    request_body = StudentIdBatchImport,
    responses(
    )
)]
pub async fn api_batch_import_student_ids(
    data: web::Data<DatabaseService>,
    req: web::Json<StudentIdBatchImport>,
) -> Result<HttpResponse> {
    info!("管理员批量导入学号");

    let response = data.batch_import_student_ids(&req.student_data, req.overwrite_existing).await;
    let http_status = if response.code == 0 { 200 } else { 400 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 更新学号信息
#[utoipa::path(
    put,
    path = "/api/v1/admin/student-ids/{id}",
    tag = "Student Management",
    params(
    ),
    request_body = UpdateStudentIdRequest,
    responses(
    )
)]
pub async fn api_update_student_id(
    data: web::Data<DatabaseService>,
    path: web::Path<i32>,
    req: web::Json<UpdateStudentIdRequest>,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    info!("管理员更新学号信息: ID {}", id);

    let response = data.update_student_id(id, req.student_name.as_deref(), req.class_info.as_deref()).await;
    let http_status = if response.code == 0 { 200 } else { 400 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 删除学号
#[utoipa::path(
    delete,
    path = "/api/v1/admin/student-ids/{id}",
    tag = "Student Management",
    params(
    ),
    responses(
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
#[utoipa::path(
    get,
    path = "/api/v1/admin/student-ids/stats",
    tag = "Student Management",
    responses(
    )
)]
pub async fn api_get_student_id_stats(
    data: web::Data<DatabaseService>,
) -> Result<HttpResponse> {
    info!("管理员请求学号统计");

    let response = data.get_student_id_stats().await;
    let http_status = if response.code == 0 { 200 } else { 500 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// DormDB API 文档
#[derive(OpenApi)]
#[openapi(
    info(
        title = "DormDB API",
        version = "1.0.0",
        contact(
            name = "DormDB Team",
            email = "admin@dormdb.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
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
    )
)]
pub struct ApiDoc;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/apply", web::post().to(apply_database))
            .route("/applicants", web::get().to(get_applicants))
            .route("/health", web::get().to(health_check))
            // 管理员接口
            .route("/admin/status", web::get().to(get_system_status))
            .route("/admin/stats", web::get().to(get_application_stats))
            .route(
                "/admin/repair",
                web::post().to(check_and_repair_consistency),
            )
            .route("/admin/login", web::post().to(admin_login))
            .route("/admin/delete", web::post().to(admin_delete_user))
            // 学号管理接口
            .route("/admin/student-ids", web::get().to(api_get_student_ids))
            .route("/admin/student-ids", web::post().to(api_add_student_id))
            .route("/admin/student-ids/batch-import", web::post().to(api_batch_import_student_ids))
            .route("/admin/student-ids/{id}", web::put().to(api_update_student_id))
            .route("/admin/student-ids/{id}", web::delete().to(api_delete_student_id))
            .route("/admin/student-ids/stats", web::get().to(api_get_student_id_stats))
            // 用户管理接口
            .route("/admin/users", web::get().to(api_get_all_users))
            .route("/admin/users/{identity_key}", web::delete().to(api_delete_user_by_identity))
            // 公开接口
            .route(
                "/public/applications",
                web::get().to(get_public_applications),
            ),
    );
}

// 用户管理 API

/// 获取所有用户列表
#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    tag = "User Management",
    responses(
        (status = 200, body = ApiResponse<Vec<UserDatabaseInfo>>),
        (status = 500, body = ApiResponse<String>)
    )
)]
pub async fn api_get_all_users(
    data: web::Data<DatabaseService>,
) -> Result<HttpResponse> {
    info!("管理员请求用户列表");

    let response = data.get_all_users().await;
    let http_status = if response.code == 0 { 200 } else { 500 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}

/// 删除用户
#[utoipa::path(
    delete,
    path = "/api/v1/admin/users/{identity_key}",
    tag = "User Management",
    params(
        ("identity_key" = String, Path, description = "用户身份标识")
    ),
    request_body = DeleteUserRequest,
    responses(
        (status = 200, body = ApiResponse<String>),
        (status = 400, body = ApiResponse<String>),
        (status = 500, body = ApiResponse<String>)
    )
)]
pub async fn api_delete_user_by_identity(
    data: web::Data<DatabaseService>,
    path: web::Path<String>,
    req: web::Json<DeleteUserRequest>,
) -> Result<HttpResponse> {
    let identity_key = path.into_inner();
    info!("管理员删除用户: {}", identity_key);

    let response = data.delete_user_by_identity(&identity_key, &req.reason).await;
    let http_status = if response.code == 0 { 200 } else { 400 };

    Ok(
        HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
            .json(response),
    )
}
