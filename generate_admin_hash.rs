#!/usr/bin/env cargo script
//! ```cargo
//! [dependencies]
//! bcrypt = "0.15"
//! ```

use std::env;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print!("请输入管理员密码: ");
        io::stdout().flush().unwrap();
        
        let mut password = String::new();
        io::stdin().read_line(&mut password).unwrap();
        let password = password.trim();
        
        if password.is_empty() {
            eprintln!("密码不能为空");
            return;
        }
        
        generate_hash(password);
    } else {
        let password = &args[1];
        generate_hash(password);
    }
}

fn generate_hash(password: &str) {
    match bcrypt::hash(password, 12) {
        Ok(hash) => {
            println!("管理员密码哈希已生成:");
            println!("ADMIN_PASSWORD_HASH={}", hash);
            println!("");
            println!("请将此哈希值设置为环境变量 ADMIN_PASSWORD_HASH");
            println!("例如：export ADMIN_PASSWORD_HASH='{}'", hash);
        }
        Err(e) => {
            eprintln!("生成哈希失败: {}", e);
        }
    }
}