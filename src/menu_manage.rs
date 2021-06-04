use crate::util;

use steam_workshop_api::{Workshop, WorkshopItem};
use prettytable::{Table, Row, Cell, row, cell};
use chrono::prelude::*;

struct UnknownFile {
    filename: String,
    size: Option<u64>,
    modified: Option<std::time::SystemTime>
}

pub fn handler(menu: &util::MenuParams) -> Result<Option<util::MenuResult>, Box<dyn std::error::Error>> {
    let mut unknownid_filenames: Vec<UnknownFile> = Vec::new();
    let fileids = match Workshop::get_vpks_in_folder(&menu.config.gamedir) {
        Ok(results) => {
            //Tries to find an ID to parse
            let mut fileids: Vec<String> = Vec::with_capacity(results.len());
            for filename in results.iter() {
                if let Some(id) = util::Regexes::get_filename_addonid(&filename) {
                    fileids.push(id);
                } else {
                    let full_file = format!("{}.vpk", filename);
                    if let Ok(metadata) = std::fs::metadata(&menu.config.gamedir.join(full_file)) {
                        unknownid_filenames.push(UnknownFile {
                            filename: filename.clone(), 
                            size: Some(metadata.len()),
                            modified: metadata.modified().ok()
                        });
                    } else {
                        unknownid_filenames.push(UnknownFile {
                            filename: filename.clone(), 
                            size: None,
                            modified: None
                        });
                    }
                }
            }
            fileids
        },
        Err(err) => {
            menu.logger.error("MenuManage/get_vpks_in_folder", &format!("Error finding vpks in \"{}\": \n{}\n", 
                &menu.config.get_game_path_str().unwrap(), 
                err
            ));
            return Ok(None)
        }
    };

    let spinner = util::setup_spinner("Getting VPK Details...");
    let details: Vec<WorkshopItem> = match menu.workshop.get_published_file_details(&fileids) {
        Ok(details) => details,
        Err(err) => { 
            spinner.abandon();
            menu.logger.error("MenuManage/get_file_details", &err.to_string());
            return Ok(None)
        }
    };

    spinner.finish_and_clear();

    println!("{}", console::style("Workshop Items").bold());
    let mut table = Table::new();
    table.set_titles(row!["Item Name", "File Size", "Last Update", "Status"]);

    for item in details {
        let mut date = chrono::Utc.timestamp_opt(item.time_updated as i64, 0);
        let status_cell = match menu.config.get_download(&item.publishedfileid) {
            Some(downloaded) => {
                date = chrono::Utc.timestamp_opt(downloaded.time_updated as i64, 0);
                if downloaded.time_updated < item.time_updated {
                    Cell::new("Update Available")
                } else {
                    Cell::new("Up-to-date")
                }
            }
            None => Cell::new("External File")
        };
        table.add_row(
            Row::new(vec![
                Cell::new(&item.title),
                Cell::new(&util::format_bytes(item.file_size)),
                Cell::new(&date.unwrap().format("%Y/%m/%d").to_string()),
                status_cell,
            ])
        );
    }
    for unknown in unknownid_filenames {
        let size_cell_text: String = match unknown.size {
            Some(size) => util::format_bytes(size),
            None => "n/a".to_owned()
        };
        let date_cell_text = match unknown.modified {
            Some(date) => {
                let date: DateTime<Local> = date.into();
                date.format("%Y/%m/%d").to_string()
            },
            None => "n/a".to_owned()
        };
        table.add_row(
            Row::new(vec![
                Cell::new(&unknown.filename),
                Cell::new(&size_cell_text),
                Cell::new(&date_cell_text),
                Cell::new("(No ID Found)"),
            ])
        );
    }
    table.printstd();
    Ok(None)
}