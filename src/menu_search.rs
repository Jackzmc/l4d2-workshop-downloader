use crate::workshop::{Workshop, WorkshopItem};
use crate::util;
use crate::meta;

use dialoguer::{Input};
use console::style;

pub fn handler(_config: &meta::Config , workshop: &Workshop) -> Result<(), Box<dyn std::error::Error>> {
    let input : String = Input::new()
        .with_prompt("Enter a search query")
        .interact_text()?;

    let spinner = util::setup_spinner("Fetching search results...");
    let results: Vec<WorkshopItem> = match workshop.search_full(550, &input) {
        Ok(results) => results,
        Err(err) => {
            println!("{} {}", 
                style("Error").bold().red(),
                style(err).red()
            );
            return Ok(())
        }
    };
    for (i, item) in results.iter().enumerate() {
        println!("{}. {}", i, item.title);
    }
    spinner.finish_and_clear();

    Ok(())
}