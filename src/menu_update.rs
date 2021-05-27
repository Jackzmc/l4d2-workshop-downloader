//TODO: Grab items in json, fetch_vpk_details() -> check if date > old date, download file_url
use std::{fs, io, path::PathBuf, error::Error, thread};
use indicatif::{HumanDuration};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{Write};
use std::clone::Clone;

use crate::{meta, util, workshop};

pub fn handler(_config: &meta::Config) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let downloads = &_config.downloads;
    let ids: Vec<String> = downloads
        .iter()
        .map(|download| download.publishedfileid.clone())
        .collect();

    let spinner = util::setup_spinner("Fetching Latest File Info...");
    let details = workshop::get_vpk_details(&client, &ids)?;
    spinner.finish_with_message("Fetched");

    let mut outdated: Vec<workshop::WorkshopItem> = Vec::new();

    for (i, entry) in downloads.into_iter().enumerate() {
        //TODO: Move '>=' to '>' once testing complete
        if details[i].time_updated >= entry.time_updated {
            let duration = std::time::Duration::from_secs(details[i].time_updated as u64 - entry.time_updated as u64);
            let hd = HumanDuration(duration);
            println!("Item \"{title}\" is out of date. {hd}", title=entry.title, hd=hd);
            outdated.push(details[i].clone());
        }
    }

    if outdated.len() == 0 {
        println!("No items need updating.\n");
        return Ok(())
    }

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Updating {} workshop items, continue?", outdated.len()))
        .default(true)
        .interact()
        .unwrap()
    {
        let directory = _config.get_game_path();

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
            

            for byte in response.bytes() {
                match dest.write_all(&byte) {
                    Ok(()) => pb.inc(byte.len() as u64),
                    _ => pb.finish_with_message(format!("Updated {}", item.title))
                }
            }
        }
        spinner.finish();
    } else {
        println!("Update was cancelled. Returning to menu.");
    }
    Ok(())
}