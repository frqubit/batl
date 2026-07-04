use colored::*;
use std::sync::atomic::{AtomicBool, Ordering};

static FORCE_QUIET: AtomicBool = AtomicBool::new(false);

pub fn force_quiet_behavior() {
    FORCE_QUIET.store(true, Ordering::Relaxed);
}

pub fn success(message: &str) {
    if !FORCE_QUIET.load(Ordering::Relaxed) {
        println!("[{}] {}", "OK".green(), message)
    }
}

pub fn error(message: &str) {
    if !FORCE_QUIET.load(Ordering::Relaxed) {
        println!("[{}] {}", "ERR".red(), message)
    }
}

pub fn warn(message: &str) {
    if !FORCE_QUIET.load(Ordering::Relaxed) {
        println!("[{}] {}", "WARN".yellow(), message)
    }
}

pub fn info(message: &str) {
    if !FORCE_QUIET.load(Ordering::Relaxed) {
        println!("[{}] {}", "INFO".blue(), message)
    }
}
