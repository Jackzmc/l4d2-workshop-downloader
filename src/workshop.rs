use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf, collections::HashMap};


#[derive(Serialize, Deserialize)]
pub struct WorkshopItem {
    pub publishedfileid: String,
    result: i8,
    creator: String,
    creator_app_id: u32,
    consumer_app_id: u32,
    pub filename: String,
    pub file_size: usize,
    pub file_url: String,
    preview_url: String,
    hcontent_preview: String,
    pub title: String,
    pub description: String,
    pub time_created: usize,
    pub time_updated: usize,
    subscriptions: u32,
    favorited: u32,
    views: u32,
    tags: Vec<WorkshopItemTag>
}

#[derive(Serialize, Deserialize)]
pub struct WorkshopItemTag {
    tag: String
}

#[derive(Serialize, Deserialize)]
struct WSResponse {
    response: WSResponseBody
}

#[derive(Serialize, Deserialize)]
struct WSResponseBody {
    publishedfiledetails: Vec<WorkshopItem>
}

pub fn get_vpks(dir: &str) -> Result<Vec<String>, String> {
    let mut entries: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(file) => {
            match file.map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>() {
                Ok(files) => files,
                Err(err) => return Err(err.to_string())
            }
        },
        Err(err) => return Err(err.to_string())
    };

    // The order in which `read_dir` returns entries is not guaranteed. If reproducible
    // ordering is required the entries should be explicitly sorted.

    entries.sort();

    let mut vpks: Vec<String> = Vec::new();

    for entry in entries {
        if !entry.is_dir() {
            if entry.extension().unwrap() == "vpk" {
                vpks.push(entry.file_stem().unwrap().to_str().unwrap().to_owned());
            }
        }
    }
    
    Ok(vpks)
}

pub fn get_vpk_details(client: reqwest::blocking::Client, vpks: &[String]) -> Result<Vec<WorkshopItem>, Box<dyn std::error::Error>> {
    let mut params = HashMap::new();
    let length = vpks.len().to_string();
    params.insert("itemcount".to_string(), length);
    for (i, vpk) in vpks.iter().enumerate() {
        let name = format!("publishedfileids[{i}]", i=i);
        params.insert(name, vpk.to_string());
    }
    let details = client.post("https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/")
        .header("User-Agent", "L4D2-Workshop-Downloader")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()?
        .json::<WSResponse>()?;

    let mut details_final: Vec<WorkshopItem> = Vec::new();

    for detail in details.response.publishedfiledetails {
        details_final.push(detail);
    }

    Ok(details_final)
}

pub fn save_vpk() {

}