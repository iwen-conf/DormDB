use log::{info, warn};
use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::{HttpRequest, Result, web};

const ROOT_PAYLOAD_PATH: &str = "static/_payload.json";

fn route_payload_path(req: &HttpRequest) -> PathBuf {
    let request_path = req.path();

    if request_path.starts_with("/admin/") {
        let candidate = PathBuf::from("static")
            .join(request_path.trim_start_matches('/'))
            .join("_payload.json");
        if candidate.exists() {
            return candidate;
        }
    }

    if request_path.starts_with("/user/") {
        let candidate = PathBuf::from("static")
            .join(request_path.trim_start_matches('/'))
            .join("_payload.json");
        if candidate.exists() {
            return candidate;
        }
    }

    PathBuf::from(ROOT_PAYLOAD_PATH)
}

/// Payload 文件处理器
/// 返回 Nuxt 的 payload JSON 文件，支持查询参数
pub async fn payload_handler(req: HttpRequest) -> Result<NamedFile> {
    let payload_path = route_payload_path(&req);
    let debug_enabled = std::env::var("DEBUG_UI_UX_FIX")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if debug_enabled {
        info!(
            "DEBUG: payload_handler path={} resolved={} query={}",
            req.path(),
            payload_path.display(),
            req.query_string()
        );
    } else {
        info!("返回 payload 文件: {}", payload_path.display());
    }

    Ok(NamedFile::open(payload_path)?)
}

/// SPA 回退处理器
/// 处理前端路由，如果文件不存在则回退到 200.html
pub async fn spa_handler(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req
        .match_info()
        .query("filename")
        .parse()
        .unwrap_or_default();
    if req.path().contains("_payload.json") {
        let payload_path = route_payload_path(&req);
        return Ok(NamedFile::open(payload_path)?);
    }

    let file_path = PathBuf::from("static").join(&path);

    info!("SPA处理器 - 请求路径: {:?}", file_path);

    // 如果文件存在，直接返回
    if file_path.exists() && file_path.is_file() {
        info!("返回文件: {:?}", file_path);
        return Ok(NamedFile::open(file_path)?);
    }

    // 如果是目录，尝试返回 index.html
    let index_path = file_path.join("index.html");
    if index_path.exists() {
        info!("返回目录索引: {:?}", index_path);
        return Ok(NamedFile::open(index_path)?);
    }

    // SPA 回退到 200.html
    let fallback_path = PathBuf::from("static/200.html");
    info!("文件不存在，SPA回退到: {:?}", fallback_path);
    Ok(NamedFile::open(fallback_path)?)
}

/// 根路径处理器
/// 返回主页 (学生申请页面)
pub async fn index_handler() -> Result<NamedFile> {
    info!("访问主页");
    Ok(NamedFile::open("static/index.html")?)
}

/// 管理员页面处理器
/// 处理 /admin/* 路径的请求
pub async fn admin_handler(path: web::Path<String>) -> Result<NamedFile> {
    let admin_path = path.as_str();
    info!("访问管理员页面: {}", admin_path);

    // 构建文件路径
    let file_path = match admin_path {
        "" | "/" => PathBuf::from("static/admin/login/index.html"), // 默认到登录页
        "login" => PathBuf::from("static/admin/login/index.html"),
        "dashboard" => PathBuf::from("static/admin/dashboard/index.html"),
        "students" => PathBuf::from("static/admin/students/index.html"),
        _ => {
            // 尝试直接访问路径
            let direct_path = PathBuf::from(format!("static/admin/{}/index.html", admin_path));
            if direct_path.exists() {
                direct_path
            } else {
                // 回退到登录页
                PathBuf::from("static/admin/login/index.html")
            }
        }
    };

    if file_path.exists() {
        info!("返回管理员页面: {:?}", file_path);
        Ok(NamedFile::open(file_path)?)
    } else {
        // 最终回退到登录页
        warn!("管理员页面不存在，回退到登录页");
        Ok(NamedFile::open("static/admin/login/index.html")?)
    }
}

/// 404 错误处理器
#[allow(dead_code)]
pub async fn not_found_handler() -> Result<NamedFile> {
    warn!("返回404页面");
    Ok(NamedFile::open("static/404.html")?)
}

/// 配置静态文件路由
pub fn configure_static_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // 根路径
        .route("/", web::get().to(index_handler))
        // 静态资源文件服务 (优先级最高)
        .service(
            actix_files::Files::new("/_nuxt", "static/_nuxt")
                .show_files_listing()
                // 开发环境禁用缓存，避免前端重新生成时的缓存问题
                .use_etag(false)
                .use_last_modified(false),
        )
        .service(
            actix_files::Files::new("/favicon.ico", "static/favicon.ico")
                .use_etag(false)
                .use_last_modified(false),
        )
        .service(
            actix_files::Files::new("/robots.txt", "static/robots.txt")
                .use_etag(false)
                .use_last_modified(false),
        )
        // payload 文件处理 (支持查询参数)
        .route("/_payload.json", web::get().to(payload_handler))
        .route("/admin/login/_payload.json", web::get().to(payload_handler))
        .route(
            "/admin/dashboard/_payload.json",
            web::get().to(payload_handler),
        )
        .route(
            "/admin/students/_payload.json",
            web::get().to(payload_handler),
        )
        .route(
            "/user/profile/_payload.json",
            web::get().to(payload_handler),
        )
        // 统一的前端视觉增强样式
        .service(
            actix_files::Files::new("/assets", "static/assets")
                .use_etag(false)
                .use_last_modified(false),
        )
        // 管理员页面路由 (具体路径优先)
        .route(
            "/admin",
            web::get().to(|| async { admin_handler(web::Path::from("".to_string())).await }),
        )
        .route(
            "/admin/",
            web::get().to(|| async { admin_handler(web::Path::from("".to_string())).await }),
        )
        .route(
            "/admin/login",
            web::get().to(|| async { admin_handler(web::Path::from("login".to_string())).await }),
        )
        .route(
            "/admin/dashboard",
            web::get()
                .to(|| async { admin_handler(web::Path::from("dashboard".to_string())).await }),
        )
        .route(
            "/admin/students",
            web::get()
                .to(|| async { admin_handler(web::Path::from("students".to_string())).await }),
        )
        // 通配符路由 (放在最后)
        .route("/admin/{path:.*}", web::get().to(admin_handler))
        // SPA 回退处理 (放在最后，捕获所有其他路径)
        .default_service(web::get().to(spa_handler));
}
