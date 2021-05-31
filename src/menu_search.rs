use crate::util;
use crate::meta;

use console::style;
use steamwebapi::{Workshop, WorkshopSearchItem};
use dialoguer::{theme::ColorfulTheme, Select, Input};


pub fn handler(_config: &meta::Config, workshop: &Workshop) -> Result<Option<util::MenuResult>, Box<dyn std::error::Error>> {
    let input : String = Input::new()
        .with_prompt("Enter a search query")
        .interact_text()?;

    //let spinner = util::setup_spinner("Fetching search results...");
    match workshop.search_proxy_full(550, &input, 10) {
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
            match prompt_choose_item(&items, &itms_dis) {
                ItemResult::SearchSame => prompt_choose_item(&items, &itms_dis),
                ItemResult::SearchAnother => return handler(_config, workshop),
                _ => return Ok(None)
            };
        },
        Err(err) => eprintln!("{} {}", 
            style("Error:").bold().red(),
            style(err).red()
        )
    }
    //spinner.finish_and_clear();
    Ok(None)
}

fn prompt_choose_item(items: &[WorkshopSearchItem], itms_dis: &[String]) -> ItemResult {
    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Search Results ({} items, page {})", items.len(), 1))
        .items(&itms_dis)
        .interact()
    {
        Ok(index) => {
            if index > itms_dis.len() {
                return ItemResult::None
            }
            return print_item(&items[index])
        },
        Err(err) => eprintln!("{} {}", 
            style("Error:").bold().red(),
            style(err).red()
        )
    }
    return ItemResult::None;
}

fn print_item(item: &steamwebapi::WorkshopSearchItem) -> ItemResult {
    println!();
    println!("{}", style(&item.title).bold().underlined());
    println!("{} views\t{} favorites\t{} subscriptions", &item.views, &item.favorited, &item.subscriptions);
    println!("Created {}\tLast Updated {}", &item.time_created, &item.time_updated);
    println!();
    println!("{}", &item.file_description);

    prompt_item_options(item)
}

enum ItemResult {
    SearchAnother,
    SearchSame,
    None
}

fn prompt_item_options(item: &steamwebapi::WorkshopSearchItem) -> ItemResult {
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
        Err(err) => eprintln!("{} {}", 
            style("Error:").bold().red(),
            style(err).red()
        )
    }
    return ItemResult::None
}