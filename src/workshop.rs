use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf, path::Path, collections::HashMap, fmt};
use reqwest::blocking::Client;

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

impl fmt::Display for WorkshopItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.title, self.publishedfileid)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkshopItemTag {
    tag: String
}

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSResponse<T> {
    response: WSResponseBody<T>
}

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSResponseBody<T> {
    publishedfiledetails: Vec<T>
}

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
struct WSSearchBody {
    result: u8,
    publishedfileid: String,
    language: u8
}

#[derive(Serialize, Deserialize)]
pub struct DownloadEntry {
    pub title: String,
    pub publishedfileid: String,
    pub time_updated: usize
}

pub struct Workshop {
    client: Client,
    apikey: Option<String>,
    use_proxy: bool
}

impl Workshop {
    pub fn new(client: Option<Client>) -> Workshop {
        let client = match client {
            Some(client) => client,
            None => reqwest::blocking::Client::new()
        };
        Workshop {
            client: client,
            apikey: None,
            use_proxy: false
        }
    }

    pub fn set_apikey<'a>(&'a mut self, apikey: String) -> &'a mut Workshop {
        self.apikey = Some(apikey);
        self
    }

    pub fn use_proxy<'a>(&'a mut self, value: bool) -> &'a mut Workshop {
        self.use_proxy = value;
        self
    }

    /// Gets all *.vpk files in a directory
    pub fn get_vpks_in_folder(dir: &Path) -> Result<Vec<String>, String> {
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
                if let Some("vpk") = entry.extension().and_then(std::ffi::OsStr::to_str) {
                    vpks.push(entry.file_stem().unwrap().to_str().unwrap().to_owned())
                }
            }
        }
        
        Ok(vpks)
    }

    /// Fetches the latest WorkshopItem per each addon id
    pub fn get_file_details(&self, fileids: &[String]) -> Result<Vec<WorkshopItem>, Box<dyn std::error::Error>> {
        let mut params = HashMap::new();
        let length = fileids.len().to_string();
        params.insert("itemcount".to_string(), length);
        for (i, vpk) in fileids.iter().enumerate() {
            let name = format!("publishedfileids[{i}]", i=i);
            params.insert(name, vpk.to_string());
        }
        let details: WSResponse<WorkshopItem> = self.client
            .post("https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/")
            .header("User-Agent", format!("L4D2-Workshop-Downloader/v{}", env!("CARGO_PKG_VERSION")))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()?
            .json::<WSResponse<WorkshopItem>>()?;
    
        let mut details_final: Vec<WorkshopItem> = Vec::new();
    
        for detail in details.response.publishedfiledetails {
            details_final.push(detail);
        }
    
        Ok(details_final)
    }

    //TODO: Extract into builder
    ///Search for workshop items
    pub fn search_ids(&self, appid: u64, query: &str) -> Result<Vec<String>, reqwest::Error> {
        if let None = &self.apikey {
            panic!("No Steam Web API key was specified");
        }

        let details = &self.client.get("https://api.steampowered.com/IPublishedFileService/QueryFiles/v1/?")
            .header("User-Agent", format!("L4D2-Workshop-Downloader/v{}", env!("CARGO_PKG_VERSION")))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&[
                ("page", "1"),
                ("numperpage", "20"),
                ("search_text", query),
                ("appid", &appid.to_string()),
                ("key", &self.apikey.as_ref().unwrap()),
            ])
            .send()?
            .json::<WSResponse<WSSearchBody>>()?;

        let mut fileids: Vec<String> = Vec::new();

        for res in &details.response.publishedfiledetails {
            fileids.push(res.publishedfileid.to_string());
        }
        Ok(fileids)
    }

    pub fn search_full(&self, appid: u64, query: &str) -> Result<Vec<WorkshopItem>, reqwest::Error> {
        if let None = &self.apikey {
            panic!("No Steam Web API key was specified");
        }

        let details = &self.client.get("https://api.steampowered.com/IPublishedFileService/QueryFiles/v1/?")
            .header("User-Agent", format!("L4D2-Workshop-Downloader/v{}", env!("CARGO_PKG_VERSION")))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .query(&[
                ("page", "1"),
                ("numperpage", "20"),
                ("search_text", query),
                ("appid", &appid.to_string()),
                ("return_metadata", "1"),
                ("key", &self.apikey.as_ref().unwrap()),
            ])
            .send()?
            .json::<WSResponse<WorkshopItem>>()?;

        Ok(details.response.publishedfiledetails.clone())
    }
}