use crate::workshop;

use std::{path::PathBuf, io, fs, env};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub gamedir: PathBuf,
    pub apikey: Option<String>,
    pub use_proxy_instead: bool,
    pub downloads: Vec<workshop::DownloadEntry>,
    pub include_name: bool
}

impl Config {
    pub fn get_game_path_str(&self) -> Option<&str> {
        self.gamedir.to_str()
    }
    pub fn format_file(&self, item: &workshop::WorkshopItem) -> String {
        if self.include_name {
            format!("{} = {}", title=item.title, id=item.publishedfileid)
        }else{
            item.publishedfileid.to_string()
        }
    }
    pub fn new(path: PathBuf) -> Config {
        Config {
            gamedir: path,
            apikey: None,
            use_proxy_instead: false,
            downloads: Vec::new(),
            include_name: true
        }
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