use crate::workshop;
use crate::util;
use crate::meta;


use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use std::{fs};

const MAX_ITEMS_PER_PAGE: usize = 20;


pub fn handler(_config: &meta::Config) -> Result<(), Box<dyn std::error::Error>> {
    //Fetch the current vpks in the workshop directory
    let spinner = util::setup_spinner("Fetching VPKS...");
    let fileids = workshop::get_vpk_ids(&_config.gamedir.join("workshop"))?;
    spinner.finish_with_message("Fetched VPKs");

    if fileids.is_empty() {
        println!("Import complete: No items were to be imported.");
        return Ok(())
    }

    //Fetch the workshop details for the vpks
    let client = reqwest::blocking::Client::new();
    let spinner = util::setup_spinner("Getting VPK Details...");
    let details = workshop::get_file_details(&client, &fileids)?;
    spinner.finish_with_message("Fetched VPK Details");

    //Setup the list of selected vpks to import, pagination
    let mut selected_vpks: Vec<workshop::DownloadEntry> = Vec::new();
    let mut page_items: Vec<String> = Vec::new();
    let size = fileids.len();
    page_items.reserve(MAX_ITEMS_PER_PAGE);
    selected_vpks.reserve(size);
    let pages = (size as f32 / MAX_ITEMS_PER_PAGE as f32).ceil() as usize;

    let defaults = vec![true; size];

    //Pagination to show MAX_ITEMS_PER_PAGE
    for page in 0..pages {
        page_items.clear();
        let start_val = page * MAX_ITEMS_PER_PAGE;
        //Add items to the page
        for (i, item) in details.iter().enumerate().skip(start_val).take(MAX_ITEMS_PER_PAGE) {
            page_items.push(format!("{i}. {title} - {id}", i=i+1, title=item.title, id=item.publishedfileid))
        }
        //Get the selection
        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Select Workshop Maps (Page {})", page + 1))
            .items(&page_items)
            .defaults(&defaults)
            .interact()
            .unwrap();
        
        //Save the meta needed to update items later
        for i in selections {
            let item = &details[i];
            let download = workshop::DownloadEntry {
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
        //Finally, write the meta info
        let dest_folder = _config.gamedir.clone();
        fs::write(dest_folder.join("downloads.json"), serde_json::to_string(&selected_vpks)?)?;
        //Loop each selected item and move it down a directory (addons/workshop -> addons/)
        for item in selected_vpks {
            fs::rename(dest_folder.join(format!("workshop/{}.vpk", item.publishedfileid)), dest_folder.join(format!("{}.vpk", item.publishedfileid)))?;
        }
        println!("Succesfully imported {} files", item_count);
    } else {
        println!("Import was cancelled.");
    }

    Ok(())
}