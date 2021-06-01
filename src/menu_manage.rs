use crate::meta;
use crate::util;

use steamwebapi::{Workshop, WorkshopItem};
use regex::Regex;
use lazy_static::lazy_static;
use prettytable::{Table, Row, Cell, row, cell};

pub fn handler(config: &meta::Config, workshop: &Workshop) -> Result<Option<util::MenuResult>, Box<dyn std::error::Error>> {
    lazy_static! {
        static ref FILENAME_ID_REG: Regex = Regex::new(r"([0-9]{7,})").unwrap();
    }
    let fileids = match Workshop::get_vpks_in_folder(&config.gamedir) {
        Ok(results) => {
            //Tries to find an ID to parse
            let mut fileids: Vec<String> = Vec::with_capacity(results.len());
            for filename in results.iter() {
                if let Some(mat) = FILENAME_ID_REG.find(&filename) {
                    let id = &filename[mat.start()..mat.end()];
                    fileids.push(id.to_string());
                }
            }
            fileids
        },
        Err(err) => {
            eprintln!("Failed to find VPKs in folder \"{}\": \n{}\n", &config.get_game_path_str().unwrap(), err);
            return Ok(None)
        }
    };

    let spinner = util::setup_spinner("Getting VPK Details...");
    let details: Vec<WorkshopItem> = match workshop.get_file_details(&fileids) {
        Ok(details) => details,
        Err(err) => { 
            spinner.abandon();
            eprintln!("{} {}", 
                console::style("Error:").bold().red(),
                console::style(err).red()
            );
            return Ok(None)
        }
    };

    spinner.finish_and_clear();

    println!("{}", console::style("Workshop Items").bold());
    let mut table = Table::new();
    table.add_row(row!["Item Name", "File Size", "Status"]);

    for item in details {
        let status_cell = match config.get_download(&item.publishedfileid) {
            Some(downloaded) => {
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
                status_cell
            ])
        );
    }
    table.printstd();
    Ok(None)
}