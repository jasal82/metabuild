use colored::Colorize;

pub fn info<S: Into<String>>(msg: S) {
    println!("[{}] {}", "INFO   ".bright_blue().bold(), msg.into());
}

#[allow(dead_code)]
pub fn warning<S: Into<String>>(msg: S) {
    println!("[{}] {}", "WARNING".bright_yellow().bold(), msg.into());
}

pub fn error<S: Into<String>>(msg: S) {
    eprintln!("[{}] {}", "ERROR  ".bright_red().bold(), msg.into());
}
