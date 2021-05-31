use crate::{meta, util};

use std::{fs};
use futures::{stream, StreamExt};
use indicatif::{HumanDuration};
use dialoguer::{theme::ColorfulTheme, Confirm};
use indicatif::{ProgressBar, MultiProgress, ProgressStyle};
use std::clone::Clone;
use steamwebapi::{Workshop, WorkshopItem};
use tokio::runtime::Runtime;
use std::io::Write;
use futures::{Future, future::BoxFuture, FutureExt};

struct Download<'a> {
    file: std::fs::File,
    url: &'a str,
    resp: Option<bytes::Bytes>,
    item: &'a WorkshopItem,
    pb: ProgressBar
}

const CONCURRENT_REQUESTS: usize = 4;
const CHUNK_SIZE: u64 = 512000;


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
    spinner.finish_and_clear();

    let mut outdated: Vec<WorkshopItem> = Vec::with_capacity(fileids.len());

    for (i, entry) in downloads.iter().enumerate() {
        //TODO: Move '>=' to '>' once testing complete
        //Check if any entry in meta is outdated
        if details[i].time_updated >= entry.time_updated {
            let duration = std::time::Duration::from_secs(details[i].time_updated as u64 - entry.time_updated as u64);
            if duration.as_secs() < 1800 {
                println!("Item \"{title}\" is out of date. Last updated recently", title=entry.title);
            }else {
                let hd = HumanDuration(duration);
                println!("Item \"{title}\" is out of date. Last update was {hd} ago.", title=entry.title, hd=hd);
            }
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

        let mut downloads: Vec<Download> = Vec::with_capacity(outdated.len());
        std::io::stderr().flush().ok();
        std::io::stdout().flush().ok();

        let mpb = MultiProgress::new();
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} {msg} - [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-");

        for item in outdated.iter().take(9) {
            
            let dest = {
                let fname = directory.join(format!("{}.vpk", item.publishedfileid));
                fs::File::create(fname).expect("Could not create file")
            };
            let pb = ProgressBar::new(item.file_size as u64)
                .with_message(format!("{} ({}.vpk)", &item.title, &item.publishedfileid))
                .with_style(style.clone());
            let download = Download {
                file: dest,
                url: &item.file_url,
                resp: None,
                item: &item,
                pb: mpb.add(pb)
            };
            downloads.push(download);
        }
        mpb.set_move_cursor(true);
        mpb.set_draw_target(indicatif::ProgressDrawTarget::stdout_with_hz(2000));

        let rt = Runtime::new()?;
        rt.block_on(async {

            //println!("> Starting download");
            stream::iter(downloads)
            .map(|mut download: Download| {
                let client = &client;
                async move {
                    //println!("Downloading {}", &download.item);
                    download.pb.println("item.");
                    match client
                        .get("http://mc.jackz.me/100MB.zip")
                        .header("User-Agent", "L4D2-Workshop-Downloader")
                        .send()
                        .await
                        
                    {
                        Ok(response) => {
                            match response.bytes().await {
                                Ok(bytes) => {
                                    for chunk in bytes.chunks_exact(CHUNK_SIZE as usize) {
                                        //download.file.write(&chunk).expect("write err");
                                        download.pb.inc(CHUNK_SIZE);
                                        download.pb.println("chunk dl");
                                        download.pb.tick();
                                    }
                                    download.resp = Some(bytes);
                                },
                                Err(err) => eprintln!("Item {} failed: {}", &download.item, err)
                            }
                            //download.pb.finish()
                        },
                        Err(err) => {
                            //download.pb.finish_with_message(format!("Item {} failed: {}", &download.item, err));
                        }
                    }
                    download.pb.finish();
                    download
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS)
            .for_each(|download| async {
                match download.resp {
                    Some(resp) => {
                        println!("Downloaded {} bytes for {}", resp.len(), &download.item);
                    },
                    None => println!("item empty {}", &download.item)
                }
                mpb.remove(&download.pb);
            })
            .await;
            tokio::task::spawn_blocking(move || mpb.join().expect("Progress bar failure"));
        });
        println!("COMPLETE!");
    } else {
        println!("Update was cancelled. Returning to menu.");
    }
    Ok(None)
}