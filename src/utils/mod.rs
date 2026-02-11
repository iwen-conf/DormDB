use rand::Rng;
use rand::seq::SliceRandom;

pub fn generate_secure_password(length: usize) -> String {
    let mut rng = rand::thread_rng();

    // 定义字符集
    let lowercase = "abcdefghijklmnopqrstuvwxyz";
    let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let numbers = "0123456789";
    let symbols = "!@#$%^&*";

    // 确保密码包含各种字符类型
    let mut password = vec![
        // 至少包含一个小写字母
        lowercase
            .chars()
            .nth(rng.gen_range(0..lowercase.len()))
            .unwrap(),
        // 至少包含一个大写字母
        uppercase
            .chars()
            .nth(rng.gen_range(0..uppercase.len()))
            .unwrap(),
        // 至少包含一个数字
        numbers
            .chars()
            .nth(rng.gen_range(0..numbers.len()))
            .unwrap(),
        // 至少包含一个符号
        symbols
            .chars()
            .nth(rng.gen_range(0..symbols.len()))
            .unwrap(),
    ];

    // 填充剩余长度
    let all_chars = format!("{}{}{}{}", lowercase, uppercase, numbers, symbols);
    for _ in 4..length {
        password.push(
            all_chars
                .chars()
                .nth(rng.gen_range(0..all_chars.len()))
                .unwrap(),
        );
    }

    // 随机打乱密码字符顺序
    password.shuffle(&mut rng);

    password.into_iter().collect()
}

pub fn validate_identity_key(identity_key: &str) -> bool {
    // 检查身份标识是否为空或过长
    if identity_key.is_empty() || identity_key.len() > 50 {
        return false;
    }

    // 检查是否只包含字母数字和下划线
    // 注意：身份标识（如学号）可以以数字开头，这与数据库标识符规则不同
    identity_key
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_password() {
        let password = generate_secure_password(16);
        assert_eq!(password.len(), 16);

        // 检查是否包含各种字符类型
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_symbol = password.chars().any(|c| "!@#$%^&*".contains(c));

        assert!(has_lowercase);
        assert!(has_uppercase);
        assert!(has_digit);
        assert!(has_symbol);
    }

    #[test]
    fn test_validate_identity_key() {
        assert!(validate_identity_key("20250701"));
        assert!(validate_identity_key("user_123"));
        assert!(validate_identity_key("abc123"));

        assert!(!validate_identity_key(""));
        assert!(!validate_identity_key("user@domain"));
        assert!(!validate_identity_key("user-123"));
        assert!(!validate_identity_key("user 123"));
    }
}
