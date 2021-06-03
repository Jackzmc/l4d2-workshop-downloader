use crate::{meta, util};
use crate::logger::LogLevel;

use std::{fs};
use futures::{stream, StreamExt};
use indicatif::{HumanDuration};
use dialoguer::{theme::ColorfulTheme, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use std::clone::Clone;
use steamwebapi::{WorkshopItem};
use tokio::runtime::Runtime;
use std::io::Write;
use console::style;

struct Download {
    file: std::fs::File,
    success: bool,
    item: WorkshopItem,
}

const CONCURRENT_REQUESTS: usize = 4;

pub fn handler(menu: &mut util::MenuParams) -> Result<Option<util::MenuResult>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    //Get downloads from meta file & check if any
    let downloads = &menu.config.downloads;
    if downloads.is_empty() {
        println!("There are no items to update.");
        return Ok(None)
    }

    //Get a array of addon ids
    let fileids: Vec<String> = downloads
        .iter()
        .map(|download| download.publishedfileid.clone())
        .collect();
    
    //Using above list, get the latest workshop info (key is time_updated)
    let spinner = util::setup_spinner("Fetching Latest File Info...");
    let details: Vec<WorkshopItem> = match menu.workshop.get_file_details(&fileids) {
        Ok(details) => details,
        Err(err) => { 
            spinner.abandon();
            menu.logger.error("MenuUpdate/get_file_details", &err.to_string());
            return Ok(None)
        }
    };
    spinner.finish_and_clear();

    let mut outdated: Vec<WorkshopItem> = Vec::with_capacity(fileids.len());

    for (i, entry) in downloads.iter().enumerate() {
        //Check if any entry in meta is outdated
        if details[i].time_updated > entry.time_updated {
            let duration = std::time::Duration::from_secs(details[i].time_updated as u64 - entry.time_updated as u64);
            let hd = HumanDuration(duration);
            println!("{title} is out of date. Last update was {hd} ago.", 
                title=style(&entry.title).bold(), 
                hd=hd);
            outdated.push(details[i].clone());
        }
    }

    if outdated.is_empty() {
        println!("All {} addons are up-to-date.", details.len());
        return Ok(None)
    }

    let items = outdated.len();
    println!();
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Are you sure you want to update {} workshop items?", items))
        .default(true)
        .interact()
        .unwrap()
    {
        let directory = menu.config.gamedir.clone();

        let mut downloads: Vec<Download> = Vec::with_capacity(items);

        println!("Downloading {} items at a time", CONCURRENT_REQUESTS);
        println!();

        let progress = ProgressBar::new(items as u64)
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

        for item in outdated {
            
            let dest = {
                let fname = directory.join(format!("{}.vpk", item.publishedfileid));
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
                                            style(format!("Download for {} failed:\n", &download.item.title)).red().bold(),
                                            style(err).red()
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

                menu.config.update_download(meta::DownloadEntry::from_item(&download.item));
                menu.config.save().ok();

                async move {
                    pb.println(format!("Updated {} as {}.vpk", &download.item.title, &download.item.publishedfileid));
                }
            })
            .await;
            progress.finish_and_clear();

        });
        println!("{}", console::style(format!("{} items successfully updated.", items)).bold());
        menu.logger.logp(LogLevel::INFO, "MenuUpdate", &format!("{} items successfully updated", items));
    } else {
        println!("Update was cancelled. Returning to menu.");
    }
    Ok(None)
}