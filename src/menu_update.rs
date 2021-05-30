use crate::{meta, util};

use std::{fs};
use futures::{stream, StreamExt};
use indicatif::{HumanDuration};
use dialoguer::{theme::ColorfulTheme, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use std::clone::Clone;
use steamwebapi::{Workshop, WorkshopItem};
use tokio::runtime::Runtime;

const CONCURRENT_REQUESTS: usize = 4;

pub fn handler(config: &meta::Config, workshop: &Workshop) -> Result<Option<util::MenuResult>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    //Get downloads from meta file & check if any
    let downloads = &config.downloads;
    if config.downloads.is_empty() {
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
    let details = workshop.get_file_details(&fileids).expect("Failed to get VPK details");
    spinner.finish_with_message("Fetched");

    let mut outdated: Vec<WorkshopItem> = Vec::with_capacity(fileids.len());

    for (i, entry) in downloads.iter().enumerate() {
        //TODO: Move '>=' to '>' once testing complete
        //Check if any entry in meta is outdated
        if details[i].time_updated >= entry.time_updated {
            let duration = std::time::Duration::from_secs(details[i].time_updated as u64 - entry.time_updated as u64);
            let hd = HumanDuration(duration);
            println!("Item \"{title}\" is out of date. {hd}", title=entry.title, hd=hd);
            outdated.push(details[i].clone());
        }
    }

    if outdated.is_empty() {
        println!("No items need updating.\n");
        return Ok(None)
    }

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Updating {} workshop items, continue?", outdated.len()))
        .default(true)
        .interact()
        .unwrap()
    {
        let directory = config.gamedir.clone();

        struct Download {
            file: std::fs::File,
            url: String
        }
        let mut downloads: Vec<Download> = Vec::with_capacity(outdated.len());
        let rt = Runtime::new()?;
        rt.block_on(async {
            //Section is WIP, need to have || threads to download and hopefully update progress bar w/o blocking
            let spinner = util::setup_spinner(format!("Updating {} items", outdated.len()));
            for item in outdated.iter() {
                let directory = directory.clone();
                let pb: ProgressBar = ProgressBar::new(item.file_size as u64)
                    .with_message(format!("{} as {}.vpk", item.title, item.publishedfileid))
                    .with_style(ProgressStyle::default_bar()
                        .template("{spinner:.green} {msg} - [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                        .progress_chars("#>-")
                    );
                let dest = {
                    let fname = directory.join(format!("{}.vpk", item.publishedfileid));
                    fs::File::create(fname).expect("Could not create file")
                };
                let download = Download {
                    file: dest,
                    url: item.file_url.clone()
                };
                downloads.push(download);
            }

            stream::iter(downloads)
            .map(|download| {
                let client = &client;
                async move {
                    let resp = client
                        .get(download.url)
                        .header("User-Agent", "L4D2-Workshop-Downloader")
                        .send()
                        .await?;
                    resp.bytes().await
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS)
            .for_each(|b| async {
                match b {
                    Ok(b) => println!("Got {} bytes", b.len()),
                    Err(e) => eprintln!("Got an error: {}", e),
                }
            })
            .await;
            spinner.finish();
        })
    } else {
        println!("Update was cancelled. Returning to menu.");
    }
    Ok(None)
}