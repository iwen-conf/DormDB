use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use dotenv::dotenv;
use log::info;
use std::env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod api;
mod auth;
mod config;
mod database;
mod models;
mod routes;
mod services;
mod utils;

use crate::api::{ApiDoc, configure_routes};
use crate::config::AppConfig;
use crate::database::DatabaseManager;
use crate::routes::configure_static_routes;
use crate::services::DatabaseService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logger
    env_logger::init();

    // Load configuration with enhanced validation
    let config = AppConfig::from_env().unwrap_or_else(|err| {
        eprintln!("❌ 配置加载失败: {}", err);
        eprintln!("💡 请确保设置了必需的环境变量，特别是 MYSQL_PASSWORD");
        eprintln!("📖 查看 .env 文件示例或项目文档了解详细配置");
        std::process::exit(1);
    });

    // Display configuration summary
    config.display_summary();

    info!(
        "🚀 启动 DormDB 服务器: {}:{}",
        config.server.host, config.server.port
    );

    // Initialize database connection
    let db_manager = DatabaseManager::new(&config).await.unwrap_or_else(|err| {
        eprintln!("Failed to initialize database: {}", err);
        std::process::exit(1);
    });

    // Create service
    let database_service = DatabaseService::new(db_manager);

    // Setup OpenAPI
    let openapi = ApiDoc::openapi();

    // Start HTTP server
    HttpServer::new(move || {
        // 更安全的CORS配置
        let cors = if cfg!(debug_assertions) {
            // 开发环境：允许本地开发服务器
            Cors::default()
                .allowed_origin("http://localhost:3000")
                .allowed_origin("http://127.0.0.1:3000")
                .allowed_origin("http://localhost:8080")
                .allowed_origin("http://127.0.0.1:8080")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::ACCEPT,
                    actix_web::http::header::CONTENT_TYPE,
                ])
                .supports_credentials()
                .max_age(3600)
        } else {
            // 生产环境：仅允许特定域名
            let allowed_origins = env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "https://your-domain.com".to_string());

            let mut cors = Cors::default()
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::ACCEPT,
                    actix_web::http::header::CONTENT_TYPE,
                ])
                .supports_credentials()
                .max_age(3600);

            // 添加允许的源
            for origin in allowed_origins.split(',') {
                cors = cors.allowed_origin(origin.trim());
            }

            cors
        };

        App::new()
            .app_data(web::Data::new(database_service.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            // API 路由 (优先级最高)
            .configure(configure_routes)
            // Swagger UI
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            // 静态页面路由 (包含静态资源服务)
            .configure(configure_static_routes)
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}
