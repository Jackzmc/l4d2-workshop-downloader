use crate::workshop::Workshop;
use crate::util;
use crate::meta;

use dialoguer::{theme::ColorfulTheme, Input};


pub fn handler(_config: &meta::Config , workshop: &Workshop) -> Result<(), Box<dyn std::error::Error>> {
    let input : String = Input::new()
        .with_prompt("Enter a search query")
        .interact_text()?;

    let spinner = util::setup_spinner("Fetching search results...");
    let results = workshop.search(550, &input);
    spinner.finish_and_clear();

    Ok(())
}