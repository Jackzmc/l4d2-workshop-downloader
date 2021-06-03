use std::{fs,path};
use chrono::prelude::*;
use console::style;

pub struct Logger {
    file: fs::File
}

pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    SUCCESS,
    DEBUG
}

impl LogLevel {
    fn name(&self) -> &'static str {
        stringify!(&self)
    }
}

impl Logger {
    pub fn new(filepath: path::PathBuf) -> Logger {
        Logger {
            file: fs::File::create(filepath).unwrap()
        }
    }

    pub fn log(&self, level: LogLevel, msg: &'static str) {
        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%Y/%m/%d");
        write!(&self.file, "[{}] [{}] {}", 
            timestamp,
            level.name(),
            msg
        );
    }

    pub fn logp(&self, level: LogLevel, prefix: &'static str, msg: &'static str) {
        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%Y/%m/%d");
        write!(&self.file, "[{}] [{}/{}] {}", 
            timestamp,
            prefix,
            level.name(),
            msg
        );
    }

    pub fn info(&self, prefix: &'static str, msg: &'static str) {
        self.logp(LogLevel::INFO, prefix, &msg);
    }

    pub fn warn(&self, prefix: &'static str, msg: &'static str) {
        self.logp(LogLevel::WARN, prefix, &msg);
        println!("{}", style(msg).yellow());
    }

    pub fn success(&self, prefix: &'static str, msg: &'static str) {
        self.logp(LogLevel::WARN, prefix, &msg);
        println!("{}", style(msg).green());
    }

    pub fn error(&self, prefix: &'static str, msg: &'static str) {
        self.logp(LogLevel::ERROR, prefix, &msg);
        eprintln!("{} {}", style("Error: ").red().bold(), style(msg).red());
    }

    pub fn debug(&self, prefix: &'static str, msg: &'static str) {
        self.logp(LogLevel::DEBUG, prefix, &msg);
        println!("{}", style(msg).magenta());
    }
}