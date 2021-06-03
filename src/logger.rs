use std::{fs,path};
use chrono::prelude::*;
pub struct Logger {
    file: fs::File
}

pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
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

    pub fn info(&self, msg: &'static str) {
        &self.log(LogLevel::INFO, &msg);
    }

    pub fn warn(&self, msg: &'static str) {
        &self.log(LogLevel::WARN, &msg);
    }

    pub fn error(&self, msg: &'static str) {
        &self.log(LogLevel::ERROR, &msg);
    }

    pub fn debug(&self, msg: &'static str) {
        &self.log(LogLevel::DEBUG, &msg);
    }
}