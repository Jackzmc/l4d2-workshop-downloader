use indicatif::{ProgressBar, ProgressStyle};
use std::{borrow::Cow, fs, io::Write};
use regex::Regex;
use lazy_static::lazy_static;
use tokio::runtime::Runtime;
use futures::{stream, StreamExt};

use crate::meta::{Config, DownloadEntry};
use crate::logger::Logger;
use steam_workshop_api::Workshop;

pub struct MenuResult {

}

pub struct MenuParams<'a> {
    pub config: &'a mut Config,
    pub workshop: &'a Workshop,
    pub logger: &'a Logger
}

pub fn setup_spinner(msg: impl Into<Cow<'static, str>>) -> ProgressBar {
    let spinner: ProgressBar = ProgressBar::new_spinner()
        .with_message(msg);
    spinner.enable_steady_tick(1000u64);
    spinner
}

pub struct Regexes {}
impl Regexes {
    pub fn get_filename_addonid(filename: &str) -> Option<String>  {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r"([0-9]{7,})").unwrap();
        }
        if let Some(mat) = REGEX.find(&filename) {
            return Some(filename[mat.start()..mat.end()].to_string());
        } else {
            None
        }
    }
    pub fn get_id_from_workshop_url(url: &str) -> Option<String> {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r"https?://steamcommunity.com/(workshop|sharedfiles)/filedetails/\?id=([0-9]+)").unwrap();
        }
        //https://steamcommunity.com/sharedfiles/filedetails/?id=1558774205
        if let Some(caps) = REGEX.captures(url) {
            if caps.len() == 3 {
                return Some(caps[2].to_string())
            } 
        }
        None
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes > 1000000000 {
        format!("{:.1} GB", bytes as f64 / 1000000000.0)
    } else if bytes > 1000000 {
        format!("{:.1} MB", bytes as f64 / 1000000.0)
    } else if bytes > 1000 {
        format!("{:.1} KB", bytes as f64 / 1000.0)
    } else {
        format!("{} B", bytes)
    }
}


const CONCURRENT_REQUESTS: usize = 4;

struct Download {
    file: std::fs::File,
    success: bool,
    item: steam_workshop_api::WorkshopItem,
}

pub fn download_addons(menu: &mut MenuParams, items: &[steam_workshop_api::WorkshopItem]) -> Result<(), Box<dyn std::error::Error>> {
    let progress = ProgressBar::new(items.len() as u64)
    .with_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:60.cyan/blue}] {pos} / {len} items updated ({percent}%)")
        .progress_chars("#>-")
        .tick_strings(&[
            "↓    ",
            "↓ .  ",
            "↓ .. ",
            "↓ ...",
            ""
        ])
        //"―\\|/―\\|/―"
    );

    let mut downloads: Vec<Download> = Vec::with_capacity(items.len());
    for item in items {
        
        let dest = {
            let fname = menu.config.gamedir.join(format!("{}.vpk", item.publishedfileid));
            fs::File::create(fname).expect("Could not create file")
        };
        let download = Download {
            file: dest,
            item: item.clone(),
            success: false
        };
        downloads.push(download);
    }
    progress.tick();
    progress.enable_steady_tick(500);

    let client = reqwest::Client::new();

    let rt = Runtime::new()?;
    rt.block_on(async {
        stream::iter(downloads)
        .map(|mut download: Download| {
            let client = &client;
            let pb = &progress;
            async move {
                pb.set_message(format!("{}", &download.item.title));
                match client
                    .get(&download.item.file_url)
                    .header("User-Agent", "L4D2-Workshop-Downloader")
                    .send()
                    .await
                    
                {
                    Ok(response) => {
                        let mut stream = response.bytes_stream();
                        while let Some(result) = stream.next().await {
                            match result {
                                Ok(chunk) => {
                                    if let Err(err) = download.file.write(&chunk) {
                                        println!("[{}] Write Error: {}", &download.item.publishedfileid, err);
                                        break;
                                    }
                                
                                },
                                Err(err) => {
                                    println!("{}\n{}", 
                                        console::style(format!("Download for {} failed:\n", &download.item.title)).red().bold(),
                                        console::style(err).red()
                                    );
                                    break;
                                }
                            }
                        }
                        download.file.flush().ok();
                        download.success = true;
                    },
                    Err(err) => {
                        println!("Download failure for {}: {}", &download.item, err);
                    }
                }
                download
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS)
        .for_each(|download| {
            progress.inc(1);
            let pb = &progress;

            let entry = DownloadEntry::from_item(&download.item);

            match menu.config.find_download(&entry) {
                Some(index) => menu.config.downloads[index] = entry,
                None => menu.config.add_download(entry)
            }

            menu.config.save().ok();

            async move {
                pb.println(format!("Updated {} as {}.vpk", &download.item.title, &download.item.publishedfileid));
            }
        })
        .await;
        progress.finish_and_clear();

    });
    Ok(())
}