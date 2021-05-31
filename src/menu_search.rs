use steamwebapi::Workshop;
use crate::util;
use crate::meta;

use dialoguer::Input;
use console::style;

pub fn handler(_config: &meta::Config, workshop: &Workshop) -> Result<Option<util::MenuResult>, Box<dyn std::error::Error>> {
    let input : String = Input::new()
        .with_prompt("Enter a search query")
        .interact_text()?;

    //let spinner = util::setup_spinner("Fetching search results...");
    let results = workshop.search_proxy_full(550, &input, 10);
    match results {
        Ok(results) => {
            println!();
            for (i, item) in results.iter().enumerate() {
                println!("{}. {}", i, item.title);
            }
        },
        Err(err) => {
            eprintln!("{} {}", 
                style("Error:").bold().red(),
                style(err).red()
            );
        }
    }
    //spinner.finish_and_clear();
    Ok(None)
}