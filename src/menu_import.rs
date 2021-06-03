use crate::util;
use crate::meta::{DownloadEntry};
use crate::logger::LogLevel;

use steamwebapi::{Workshop, WorkshopItem};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use std::{fs};

const MAX_ITEMS_PER_PAGE: usize = 20;


pub fn handler(menu: &util::MenuParams) -> Result<Option<util::MenuResult>, Box<dyn std::error::Error>> {
    //Fetch the current vpks in the workshop directory
    let spinner = util::setup_spinner("Fetching VPKS...");
    let folder = &menu.config.gamedir.join("workshop");
    let fileids = match Workshop::get_vpks_in_folder(folder) {
        Ok(results) => results,
        Err(err) => {
            spinner.abandon();
            menu.logger.error("MenuImport/get_vpks_in_folder", format!("Error finding VPKS in \"{}\": \n{}\n", 
                &menu.config.get_game_path_str().unwrap(), 
                err
            ));
            return Ok(None)
        }
    };
    spinner.finish_and_clear();

    if fileids.is_empty() {
        println!("There are no items to be imported.");
        return Ok(None)
    }

    //Fetch the workshop details for the vpks
    let spinner = util::setup_spinner("Getting VPK Details...");
    let details: Vec<WorkshopItem> = match menu.workshop.get_file_details(&fileids) {
        Ok(details) => details,
        Err(err) => { 
            spinner.abandon();
            menu.logger.error("MenuImport/get_file_details", &err.to_string());
            return Ok(None)
        }
    };
    spinner.finish_and_clear();

    //Setup the list of selected vpks to import, pagination
    let mut selected_vpks: Vec<DownloadEntry> = Vec::with_capacity(fileids.len());
    let mut page_items: Vec<String> = Vec::with_capacity(MAX_ITEMS_PER_PAGE);
    let size = fileids.len();
    page_items.reserve(MAX_ITEMS_PER_PAGE);
    selected_vpks.reserve(size);
    let pages = (size as f32 / MAX_ITEMS_PER_PAGE as f32).ceil() as usize;

    let defaults = vec![true; MAX_ITEMS_PER_PAGE];

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
            .with_prompt(format!("Select Addons to Import (Page {})", page + 1))
            .items(&page_items)
            .defaults(&defaults)
            .interact()
            .unwrap();
        
        //Save the meta needed to update items later
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
        .with_prompt(format!("Are you sure you want to import {} / {} workshop items?", item_count, size))
        .default(true)
        .interact()
        .unwrap()
    {
        //Finally, write the meta info
        let dest_folder = menu.config.gamedir.clone();
        let src_folder = menu.config.gamedir.join("workshop");
        //Loop each selected item and move it down a directory (addons/workshop -> addons/)
        for download in selected_vpks {
            let filename = format!("{}.vpk", &download.publishedfileid);
            fs::rename(src_folder.join(&filename), dest_folder.join(&filename))?;
            menu.config.add_download(download);
        }
        match menu.config.save() {
            Ok(()) => { 
                println!("{}\n{}\n{}",
                    console::style(format!("Succesfully imported {} files", item_count)).bold(), 
                    "Unsubscribe from the imported addons or they will be loaded twice the next time you start the game.",
                    "https://steamcommunity.com/id/<your id>/myworkshopfiles/?appid=550&browsefilter=mysubscriptions and click the [Unsubscribe From All] button"
                );
                menu.logger.logp(LogLevel::SUCCESS, "MenuImport", format!("Imported {} workshop items", item_count));
            },
            Err(err) => {
                eprintln!("{} {}\n{}", 
                    console::style("Could not save imported items: ").bold().red(),
                    console::style(err).red(),
                    console::style("Please move any items back to workshop folder and try again.").italic()
                );
                menu.logger.logp(LogLevel::ERROR, "MenuImport", format!("Import failure: {}", err));
            }
        };
    } else {
        println!("Import was cancelled.");
    }

    Ok(None)
}