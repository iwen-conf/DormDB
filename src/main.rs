use actix_cors::Cors;
use actix_files as fs;
use actix_web::{App, HttpServer, middleware::Logger, web};
use dotenv::dotenv;
use log::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod api;
mod config;
mod database;
mod models;
mod services;
mod utils;

use crate::api::{ApiDoc, configure_routes};
use crate::config::AppConfig;
use crate::database::DatabaseManager;
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
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(database_service.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .configure(configure_routes)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            // 静态文件服务
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}
