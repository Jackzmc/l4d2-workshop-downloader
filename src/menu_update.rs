use crate::{meta, util};

use std::{fs};
use indicatif::{HumanDuration};
use dialoguer::{theme::ColorfulTheme, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use std::clone::Clone;
use std::io::Write;
use steamwebapi::{Workshop, WorkshopItem};


pub fn handler(config: &meta::Config, workshop: &Workshop) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    //Get downloads from meta file & check if any
    let downloads = &config.downloads;
    if config.downloads.is_empty() {
        println!("There are no items to update.");
        return Ok(())
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

    let mut outdated: Vec<WorkshopItem> = Vec::new();

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
        return Ok(())
    }

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Updating {} workshop items, continue?", outdated.len()))
        .default(true)
        .interact()
        .unwrap()
    {
        let directory = config.gamedir.clone();

        //Section is WIP, need to have || threads to download and hopefully update progress bar w/o blocking
        let spinner = util::setup_spinner(format!("Updating {} items", outdated.len()));
        for item in outdated.iter() {
            let client = client.clone();
            let directory = directory.clone();
            let pb: ProgressBar = ProgressBar::new(item.file_size as u64)
                .with_message(format!("{} as {}.vpk", item.title, item.publishedfileid))
                .with_style(ProgressStyle::default_bar()
                    .template("{spinner:.green} {msg} - [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .progress_chars("#>-")
                );
            let mut dest = {
                let fname = directory.join(format!("{}.vpk", item.publishedfileid));
                fs::File::create(fname).expect("Could not create file")
            };

            let response = client
                .get(&item.file_url)
                .header("User-Agent", "L4D2-Workshop-Downloader")
                .send().expect("Could not fetch");
            

            if let Ok(byte) = response.bytes() {
                pb.inc(byte.len() as u64);
                dest.write_all(&byte)?;
            }
            pb.finish_with_message(format!("Updated {}", item.title));
        }
        spinner.finish();
    } else {
        println!("Update was cancelled. Returning to menu.");
    }
    Ok(())
}