use steamwebapi;

use std::{path::PathBuf, io, fs, env};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub gamedir: PathBuf,
    pub apikey: Option<String>,
    pub downloads: Vec<steamwebapi::DownloadEntry>,
    pub include_name: bool
}

#[allow(dead_code)]
impl Config {
    pub fn get_game_path_str(&self) -> Option<&str> {
        self.gamedir.to_str()
    }

    pub fn format_file(&self, item: &steamwebapi::WorkshopItem) -> String {
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
            downloads: Vec::new(),
            include_name: true
        }
    }

    pub fn add_download(&mut self, item: steamwebapi::DownloadEntry) {
        self.downloads.push(item);
    }


    pub fn load() -> Option<Config> {
        match fs::File::open(env::current_dir().unwrap().join("downloader_meta.json")) {
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

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = env::current_dir().unwrap().join("downloader_meta.json");
        fs::write(dir, serde_json::to_string(&self)?)?;
        Ok(())
    }

}

