use crate::util;

use console::style;
use steamwebapi::{WorkshopSearchItem};
use dialoguer::{theme::ColorfulTheme, Select, Input};
use prettytable::{Table, Row, Cell, row, cell};
use chrono::prelude::*;

pub fn handler(menu: &util::MenuParams) -> Result<Option<util::MenuResult>, Box<dyn std::error::Error>> {
    let input : String = Input::new()
        .with_prompt("Enter a search query or a workshop url")
        .interact()?;

    if let Some(fileid) = util::Regexes::get_id_from_workshop_url(&input) {
        let spinner = util::setup_spinner(format!("Fetching workshop item of id {}...", fileid));
        match menu.workshop.get_file_details(&[fileid]) {
            Ok(items) => {
                let item = &items[0];
                spinner.finish_and_clear();
                match menu.workshop.get_file_children_ids(&item.publishedfileid) {
                    Ok(Some(children)) => {
                        //Item is a collection of items
                        let spinner = util::setup_spinner("Fetching collection children...");
                        match menu.workshop.get_file_details(&children) {
                            Ok(cinfo) => {
                                spinner.finish_and_clear();
                                println!();
                                println!("{}", style(format!("{} - Collection", item.title)).bold());
                                let mut table = Table::new();
                                table.set_titles(row!["Item Name", "File Size", "Last Update"]);
                                let mut total_bytes = 0;
                                for child in cinfo {
                                    let date = chrono::Utc.timestamp_opt(item.time_updated as i64, 0);
                                    total_bytes += child.file_size;
                                    table.add_row(
                                        Row::new(vec![
                                            Cell::new(&child.title),
                                            Cell::new(&util::format_bytes(child.file_size)),
                                            Cell::new(&date.unwrap().format("%Y/%m/%d").to_string())
                                        ])
                                    );
                                }
                                table.add_row(row!["TOTAL", &util::format_bytes(total_bytes), ""]);
                                table.printstd();

                                //TODO: Implement downloading
                            },
                            Err(err) => {
                                spinner.finish_and_clear();
                                menu.logger.error("MenuSearch/children:get_file_details", &err.to_string());
                            }
                        }
                    },
                    Ok(None) => {
                        //Item is a single item
                    },
                    Err(err) => {
                        menu.logger.error("MenuSearch/get_file_children_ids", &err.to_string());
                    }
                }
            },
            Err(err) => { 
                spinner.abandon();
                menu.logger.error("MenuSearch/get_file_details", &err.to_string());
                return Ok(None)
            }
        }
    } else {
        match menu.workshop.search_proxy_full(550, &input, 10) {
            Ok(items) => {
                let mut i: u64 = 0;
                let mut itms_dis: Vec<String> = items.iter()
                    .map(|item| { 
                        let size = indicatif::HumanBytes(item.file_size.parse().unwrap());
                        i += 1;
                        format!("{:2}. {} [{}]", i, console::style(&item.title).blue().bright().bold(), size)
                    })
                    .collect();
                itms_dis.push(format!("{}", style("[ Cancel ]").cyan()));
                //itms_dis.push(format!("{}", style("[ Next Page ➞ ]").green()));
    
                println!();
                match prompt_choose_item(menu, &items, &itms_dis) {
                    ItemResult::SearchSame => prompt_choose_item(menu, &items, &itms_dis),
                    ItemResult::SearchAnother => return handler(menu),
                    _ => return Ok(None)
                };
            },
            Err(err) => {
                menu.logger.error("MenuSearch/search_proxy_full", &err.to_string());
            }
        }
    }

    //let spinner = util::setup_spinner("Fetching search results...");
    
    //spinner.finish_and_clear();
    Ok(None)
}

//UTIL Methods

fn prompt_choose_item(menu: &util::MenuParams, items: &[WorkshopSearchItem], itms_dis: &[String]) -> ItemResult {
    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Search Results ({} items, page {})", items.len(), 1))
        .items(&itms_dis)
        .interact()
    {
        Ok(index) => {
            if index == items.len() {
                return ItemResult::None
            }
            return print_item(menu, &items[index])
        },
        Err(err) => eprintln!("{} {}", 
            style("Error:").bold().red(),
            style(err).red()
        )
    }
    return ItemResult::None;
}

fn print_item(menu: &util::MenuParams, item: &steamwebapi::WorkshopSearchItem) -> ItemResult {
    println!();
    println!("{}", style(&item.title).bold().underlined());
    println!("{} views\t{} favorites\t{} subscriptions", &item.views, &item.favorited, &item.subscriptions);
    println!("Created {}\tLast Updated {}", &item.time_created, &item.time_updated);
    println!();
    println!("{}", &item.file_description);

    prompt_item_options(menu, item)
}

enum ItemResult {
    SearchAnother,
    SearchSame,
    None
}

fn prompt_item_options(menu: &util::MenuParams, item: &steamwebapi::WorkshopSearchItem) -> ItemResult {
    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an option")
        .default(0)
        .items(&[
            "Download Item",
            "Open item in browser",
            "Return to selection",
            "Search a new item"
        ])
        .interact()
    {
        Ok(option) => {
            match option {
                0 => {
                    println!("Can't download");
                },
                1 => {
                    webbrowser::open(&format!("https://steamcommunity.com/sharedfiles/filedetails/?id={}", &item.publishedfileid)).ok();
                },
                3 => return ItemResult::SearchSame,
                _ => return ItemResult::SearchAnother
            }
        },
        Err(err) => menu.logger.error("MenuSearch/prompt_item_options", &err.to_string())

    }
    return ItemResult::None
}