use colored::Colorize;

pub fn warning<S: Into<String>>(msg: S) {
    println!("[{}] {}", "WARNING".bright_yellow().bold(), msg.into());
}

pub fn error<S: Into<String>>(msg: S) {
    eprintln!("[{}] {}", "ERROR  ".bright_red().bold(), msg.into());
}
