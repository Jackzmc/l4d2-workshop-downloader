use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf, collections::HashMap};

#[derive(Serialize, Deserialize, Clone)]
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


#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize)]
pub struct DownloadEntry {
    pub title: String,
    pub publishedfileid: String,
    pub time_updated: usize
}

/// Gets all *.vpk files in a directory
pub fn get_vpk_ids(dir: &PathBuf) -> Result<Vec<String>, String> {
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
            match entry.extension().and_then(std::ffi::OsStr::to_str) {
                Some("vpk") => vpks.push(entry.file_stem().unwrap().to_str().unwrap().to_owned()),
                _ => {}
            }
        }
    }
    
    Ok(vpks)
}

/// Fetches the latest WorkshopItem per each addon id
pub fn get_file_details(client: &reqwest::blocking::Client, fileids: &[String]) -> Result<Vec<WorkshopItem>, Box<dyn std::error::Error>> {
    let mut params = HashMap::new();
    let length = fileids.len().to_string();
    params.insert("itemcount".to_string(), length);
    for (i, vpk) in fileids.iter().enumerate() {
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