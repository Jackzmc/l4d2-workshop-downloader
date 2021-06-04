use crate::{util};
use crate::logger::LogLevel;

use indicatif::{HumanDuration};
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::clone::Clone;
use steamwebapi::{WorkshopItem};
use console::style;

const CONCURRENT_REQUESTS: usize = 4;

pub fn handler(menu: &mut util::MenuParams) -> Result<Option<util::MenuResult>, Box<dyn std::error::Error>> {

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
    let details: Vec<WorkshopItem> = match menu.workshop.get_published_file_details(&fileids) {
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
        println!("Downloading {} items at a time", CONCURRENT_REQUESTS);
        println!();

        util::download_addons(menu, &outdated).expect("update failed critically");
        println!("{}", console::style(format!("{} items successfully updated.", items)).bold());
        menu.logger.logp(LogLevel::INFO, "MenuUpdate", &format!("{} items successfully updated", items));
    } else {
        println!("Update was cancelled. Returning to menu.");
    }
    Ok(None)
}