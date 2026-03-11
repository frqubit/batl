use colored::*;

pub fn success(message: &str) {
    println!("[{}] {}", "OK".green(), message)
}

pub fn error(message: &str) {
    println!("[{}] {}", "ERR".red(), message)
}

pub fn warn(message: &str) {
    println!("[{}] {}", "WARN".yellow(), message)
}

pub fn info(message: &str) {
    println!("[{}] {}", "INFO".blue(), message)
}
