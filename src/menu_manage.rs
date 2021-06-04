use crate::util;

use steamwebapi::{Workshop, WorkshopItem};
use prettytable::{Table, Row, Cell, row, cell};
use chrono::prelude::*;

pub fn handler(menu: &util::MenuParams) -> Result<Option<util::MenuResult>, Box<dyn std::error::Error>> {
    let fileids = match Workshop::get_vpks_in_folder(&menu.config.gamedir) {
        Ok(results) => {
            //Tries to find an ID to parse
            let mut fileids: Vec<String> = Vec::with_capacity(results.len());
            for filename in results.iter() {
                if let Some(id) = util::Regexes::get_filename_addonid(&filename) {
                    fileids.push(id);
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
    let details: Vec<WorkshopItem> = match menu.workshop.get_file_details(&fileids) {
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
                Cell::new(&format!("{:.0} MB", item.file_size as f64 * 0.000001)),
                Cell::new(&date.unwrap().format("%Y/%m/%d").to_string()),
                status_cell,
            ])
        );
    }
    table.printstd();
    Ok(None)
}