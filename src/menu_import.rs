use crate::workshop;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use indicatif::ProgressBar;
use std::{borrow::Cow, fs, path::PathBuf};
use serde::{Deserialize, Serialize};

const ADDONS_FOLDER: &str = "D:\\_temp\\rust_ws_test"; 
const DIR: &str = "D:\\_temp\\rust_ws_test\\workshop";

const MAX_ITEM_PER_PAGE: usize = 20;

#[derive(Serialize, Deserialize)]
struct DownloadEntry {
    title: String,
    publishedfileid: String,
    time_updated: usize
}

pub fn handler() -> Result<(), Box<dyn std::error::Error>> {
    let spinner = setup_spinner("Fetching VPKS...");
    let vpks = workshop::get_vpks(DIR)?;
    spinner.finish_with_message("Fetched VPKs");

    if vpks.len() == 0 {
        println!("Import complete: No items were to be imported.");
        return Ok(())
    }

    let client = reqwest::blocking::Client::new();

    let spinner = setup_spinner("Getting VPK Details...");
    let details = workshop::get_vpk_details(client, &vpks)?;
    spinner.finish_with_message("Fetched VPK Details");

    let mut selected_vpks: Vec<DownloadEntry> = Vec::new();
    let mut multiselected: Vec<String> = Vec::new();
    let size = vpks.len();
    let pages = (size as f32 / MAX_ITEM_PER_PAGE as f32).ceil() as usize;

    let defaults = vec![true; size];

    for page in 0..pages {
        multiselected.clear();
        //0*20, 1*20
        let start_val = page * MAX_ITEM_PER_PAGE;
        for i in start_val..start_val+MAX_ITEM_PER_PAGE {
            if i >= size {
                break;
            }
            let item = &details[i];
            multiselected.push(format!("{i}. {title} - {id}", i=i+1, title=item.title, id=item.publishedfileid))
        }
        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Select Workshop Maps (Page {})", page + 1))
            .items(&multiselected)
            .defaults(&defaults)
            .interact()
            .unwrap();
        

        for i in selections {
            let item = &details[i];
            let download = DownloadEntry {
                title: item.title.to_string(),
                publishedfileid: item.publishedfileid.to_string(),
                time_updated: item.time_updated
            };
            selected_vpks.push(download);
        }
    }

    let item_count = selected_vpks.len();
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Importing {} / {} workshop items, continue?", item_count, size))
        .default(true)
        .interact()
        .unwrap()
    {
        let dest_folder: PathBuf = PathBuf::from(ADDONS_FOLDER);
        fs::write(dest_folder.join("downloads.json"), serde_json::to_string(&selected_vpks)?)?;
        for item in selected_vpks {
            fs::rename(dest_folder.join(format!("workshop/{}.vpk", item.publishedfileid)), dest_folder.join(format!("{}.vpk", item.publishedfileid)))?;
        }
        println!("Succesfully imported {} files", item_count);
    } else {
        println!("Import was cancelled.");
    }

    Ok(())
}


fn setup_spinner(msg: impl Into<Cow<'static, str>>) -> ProgressBar {
    let spinner: ProgressBar = ProgressBar::new_spinner()
        .with_message(msg);
    spinner.enable_steady_tick(1000u64);
    spinner
}