use crate::workshop;

use std::{path::PathBuf, io, fs, env};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub gamedir: PathBuf,
    pub downloads: Vec<workshop::DownloadEntry>
}

impl Config {
    pub fn get_game_path_str(&self) -> Option<&str> {
        self.gamedir.to_str()
    }

    
}

pub fn get_config() -> Option<Config> {
    let dir = env::current_dir().unwrap();
    match fs::File::open(dir.join("downloader_meta.json")) {
        Ok(file) => {
            let reader = io::BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(u) => Some(u),
                Err(_e) => None
            }
        },
        Err(_e) => None
    }
    
}