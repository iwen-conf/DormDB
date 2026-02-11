use actix_web::{Error, HttpMessage, dev::ServiceRequest};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::env;

// JWT Claims结构
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // 用户ID
    pub exp: usize,         // 过期时间
    pub iat: usize,         // 签发时间
    pub role: String,       // 角色
    pub session_id: String, // 会话ID
}

// 认证服务
pub struct AuthService {
    jwt_secret: String,
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthService {
    pub fn new() -> Self {
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            warn!("未设置JWT_SECRET环境变量，使用默认密钥（不推荐用于生产环境）");
            "default_jwt_secret_change_in_production".to_string()
        });

        if jwt_secret == "default_jwt_secret_change_in_production" {
            warn!("⚠️ 警告: 正在使用默认JWT密钥，请在生产环境中更改！");
        }

        Self { jwt_secret }
    }

    /// 生成JWT令牌
    pub fn generate_token(&self, user_id: &str, role: &str) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let exp = now + Duration::hours(24); // 24小时过期

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            role: role.to_string(),
            session_id,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )?;

        info!("为用户 {} 生成了新的JWT令牌", user_id);
        Ok(token)
    }

    /// 验证JWT令牌
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    /// 验证管理员权限
    pub fn verify_admin_token(&self, token: &str) -> Result<Claims> {
        let claims = self.validate_token(token)?;

        if claims.role != "admin" {
            return Err(anyhow::anyhow!("权限不足：需要管理员权限"));
        }

        Ok(claims)
    }
}

// 管理员认证中间件
#[allow(dead_code)]
pub async fn admin_auth_middleware(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let auth_service = AuthService::new();
    let debug_enabled = std::env::var("DEBUG_UI_UX_FIX")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if debug_enabled {
        info!(
            "DEBUG: admin_auth_middleware path={} token_len={}",
            req.path(),
            credentials.token().len()
        );
    }

    match auth_service.verify_admin_token(credentials.token()) {
        Ok(claims) => {
            if debug_enabled {
                info!(
                    "DEBUG: admin_auth_middleware success sub={} role={}",
                    claims.sub, claims.role
                );
            }
            // 将用户信息存储到请求扩展中
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(e) => {
            if debug_enabled {
                info!("DEBUG: admin_auth_middleware failed error={}", e);
            }
            error!("管理员认证失败: {}", e);
            Err((actix_web::error::ErrorUnauthorized("无效的管理员令牌"), req))
        }
    }
}

// 密码工具
pub struct PasswordUtils;

impl PasswordUtils {
    /// 哈希密码
    pub fn hash_password(password: &str) -> Result<String> {
        let cost = 12; // bcrypt成本因子
        let hashed = bcrypt::hash(password, cost)?;
        Ok(hashed)
    }

    /// 验证密码
    pub fn verify_password(password: &str, hashed: &str) -> Result<bool> {
        let is_valid = bcrypt::verify(password, hashed)?;
        Ok(is_valid)
    }

    /// 生成强密码
    #[allow(dead_code)]
    pub fn generate_strong_password(length: usize) -> String {
        use rand::Rng;
        use rand::seq::SliceRandom;

        let mut rng = rand::thread_rng();
        let lowercase = "abcdefghijklmnopqrstuvwxyz";
        let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let numbers = "0123456789";
        let symbols = "!@#$%^&*()_+-=[]{}|;:,.<>?";

        let mut password = vec![
            lowercase
                .chars()
                .nth(rng.gen_range(0..lowercase.len()))
                .unwrap(),
            uppercase
                .chars()
                .nth(rng.gen_range(0..uppercase.len()))
                .unwrap(),
            numbers
                .chars()
                .nth(rng.gen_range(0..numbers.len()))
                .unwrap(),
            symbols
                .chars()
                .nth(rng.gen_range(0..symbols.len()))
                .unwrap(),
        ];

        let all_chars = format!("{}{}{}{}", lowercase, uppercase, numbers, symbols);
        for _ in 4..length {
            password.push(
                all_chars
                    .chars()
                    .nth(rng.gen_range(0..all_chars.len()))
                    .unwrap(),
            );
        }

        password.shuffle(&mut rng);
        password.into_iter().collect()
    }

    #[allow(dead_code)]
    /// 验证密码强度
    pub fn validate_password_strength(password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(anyhow::anyhow!("密码长度至少8位"));
        }

        if password.len() > 128 {
            return Err(anyhow::anyhow!("密码长度不能超过128位"));
        }

        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_symbol = password
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

        if !has_lowercase {
            return Err(anyhow::anyhow!("密码必须包含小写字母"));
        }

        if !has_uppercase {
            return Err(anyhow::anyhow!("密码必须包含大写字母"));
        }

        if !has_digit {
            return Err(anyhow::anyhow!("密码必须包含数字"));
        }

        if !has_symbol {
            return Err(anyhow::anyhow!("密码必须包含特殊字符"));
        }

        Ok(())
    }
}

// 学号验证服务
pub struct StudentValidator;

impl StudentValidator {
    /// 验证用户编号格式
    ///
    /// 现在接受任何合理的编号格式，不再限制为特定的学号格式
    pub fn validate_student_id_format(student_id: &str) -> Result<()> {
        // 编号不能为空
        if student_id.is_empty() {
            return Err(anyhow::anyhow!("编号不能为空"));
        }

        // 编号长度限制（1-50个字符）
        if student_id.len() > 50 {
            return Err(anyhow::anyhow!("编号长度不能超过50个字符"));
        }

        // 编号只能包含字母、数字、下划线和连字符
        if !student_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(anyhow::anyhow!("编号只能包含字母、数字、下划线和连字符"));
        }

        // 编号不能以特殊字符开头或结尾
        let first_char = student_id.chars().next().unwrap();
        let last_char = student_id.chars().last().unwrap();
        if !first_char.is_alphanumeric() || !last_char.is_alphanumeric() {
            return Err(anyhow::anyhow!("编号必须以字母或数字开头和结尾"));
        }

        Ok(())
    }

    /// 验证学号是否在白名单中
    #[allow(dead_code)]
    pub fn validate_student_id_whitelist(student_id: &str, whitelist: &[String]) -> Result<()> {
        if whitelist.is_empty() {
            return Err(anyhow::anyhow!("学号白名单为空，无法验证"));
        }

        if !whitelist.contains(&student_id.to_string()) {
            return Err(anyhow::anyhow!("学号不在允许的白名单中"));
        }

        Ok(())
    }
}

// 会话管理
#[allow(dead_code)]
pub struct SessionManager {
    // 在实际应用中，这应该存储在Redis或数据库中
    // 这里为了简化使用内存存储
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl SessionManager {
    pub fn new() -> Self {
        Self {}
    }

    /// 验证会话是否有效
    pub fn validate_session(&self, session_id: &str) -> Result<bool> {
        // 这里应该查询数据库或Redis来验证会话
        // 暂时返回true，实际应用中需要实现具体逻辑
        info!("验证会话: {}", session_id);
        Ok(true)
    }

    /// 销毁会话
    pub fn destroy_session(&self, session_id: &str) -> Result<()> {
        // 这里应该从数据库或Redis中删除会话
        info!("销毁会话: {}", session_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_strength_validation() {
        assert!(PasswordUtils::validate_password_strength("Abc123!@#").is_ok());
        assert!(PasswordUtils::validate_password_strength("password").is_err());
        assert!(PasswordUtils::validate_password_strength("PASSWORD123").is_err());
        assert!(PasswordUtils::validate_password_strength("123456").is_err());
    }

    #[test]
    fn test_student_id_validation() {
        // 有效的编号格式
        assert!(StudentValidator::validate_student_id_format("2023010101").is_ok()); // 传统学号
        assert!(StudentValidator::validate_student_id_format("USER123").is_ok()); // 字母数字组合
        assert!(StudentValidator::validate_student_id_format("emp_001").is_ok()); // 下划线
        assert!(StudentValidator::validate_student_id_format("ID-2024-001").is_ok()); // 连字符
        assert!(StudentValidator::validate_student_id_format("A").is_ok()); // 单字符
        assert!(StudentValidator::validate_student_id_format("123").is_ok()); // 纯数字

        // 无效的编号格式
        assert!(StudentValidator::validate_student_id_format("").is_err()); // 空字符串
        assert!(StudentValidator::validate_student_id_format("_invalid").is_err()); // 下划线开头
        assert!(StudentValidator::validate_student_id_format("invalid_").is_err()); // 下划线结尾
        assert!(StudentValidator::validate_student_id_format("-invalid").is_err()); // 连字符开头
        assert!(StudentValidator::validate_student_id_format("invalid-").is_err()); // 连字符结尾
        assert!(StudentValidator::validate_student_id_format("user@domain").is_err()); // 包含特殊字符
        assert!(StudentValidator::validate_student_id_format(&"a".repeat(51)).is_err()); // 超长
    }

    #[test]
    fn test_jwt_token_generation() {
        let auth_service = AuthService::new();
        let token = auth_service.generate_token("test_user", "admin").unwrap();
        assert!(!token.is_empty());

        let claims = auth_service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "test_user");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_flexible_user_id_formats() {
        // 测试各种有效的用户编号格式
        let valid_ids = vec![
            "USER123",
            "emp_001",
            "ID-2024-001",
            "A",
            "123",
            "2023010101", // 传统学号格式
            "student_2024",
            "ADMIN",
            "test123",
            "user-456",
            "a1",
            "Z9",
            "user_123_test",
            "ID-001-002-003",
        ];

        for id in valid_ids {
            assert!(
                StudentValidator::validate_student_id_format(id).is_ok(),
                "应该接受有效编号: {}",
                id
            );
        }

        // 测试无效的编号格式
        let long_string = "a".repeat(51);
        let invalid_ids = vec![
            "",            // 空字符串
            "_invalid",    // 下划线开头
            "invalid_",    // 下划线结尾
            "-invalid",    // 连字符开头
            "invalid-",    // 连字符结尾
            "user@domain", // 包含特殊字符
            "user space",  // 包含空格
            "user#123",    // 包含井号
            "user.123",    // 包含点号
            &long_string,  // 超长
        ];

        for id in invalid_ids {
            assert!(
                StudentValidator::validate_student_id_format(id).is_err(),
                "应该拒绝无效编号: {}",
                id
            );
        }
    }
}
