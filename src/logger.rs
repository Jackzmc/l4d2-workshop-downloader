use std::{fs,path};
use chrono::prelude::*;
use console::style;
use std::fs::OpenOptions;
use std::io::Write;

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
            file: OpenOptions::new()
                .write(true)
                .append(true)
                .open(filepath)
                .unwrap()
        }
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%Y/%m/%d");
        writeln!(&self.file, "[{}] [{}] {}", 
            timestamp,
            level.name(),
            msg
        );
    }

    pub fn logp(&self, level: LogLevel, prefix: &'static str, msg: &str) {
        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%Y/%m/%d");
        writeln!(&self.file, "[{}] [{}/{}] {}", 
            timestamp,
            prefix,
            level.name(),
            msg
        );
    }

    pub fn info(&self, prefix: &'static str, msg: &str) {
        self.logp(LogLevel::INFO, prefix, &msg);
    }

    pub fn warn(&self, prefix: &'static str, msg: &str) {
        self.logp(LogLevel::WARN, prefix, &msg);
        println!("{}", style(msg).yellow());
    }

    pub fn success(&self, prefix: &'static str, msg: &str) {
        self.logp(LogLevel::WARN, prefix, &msg);
        println!("{}", style(msg).green());
    }

    pub fn error(&self, prefix: &'static str, msg: &str) {
        self.logp(LogLevel::ERROR, prefix, &msg);
        eprintln!("{} {}", style("Error: ").red().bold(), style(msg).red());
    }

    pub fn debug(&self, prefix: &'static str, msg: &str) {
        self.logp(LogLevel::DEBUG, prefix, &msg);
        println!("{}", style(msg).magenta());
    }
}