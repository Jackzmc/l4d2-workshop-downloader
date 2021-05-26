use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use indicatif::ProgressBar;
use std::{borrow::Cow};

mod workshop;
mod menu_import;

const SELECTIONS: &'static [&'static str] = &[
    "Import Workshop VPKs",
    "Update existing VPKs",
    "Search for new item",
    "Manage Existing Items"
];


fn main() -> Result<(), Box<dyn std::error::Error>> {
    

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick a option")
        .default(0)
        .items(&SELECTIONS)
        .interact()
        .unwrap();
    if selection == 0 {
        menu_import::handler()?;
    } else {
        println!("Option not supported.");
    }
    Ok(())
}

fn setup_spinner(msg: impl Into<Cow<'static, str>>) -> ProgressBar {
    let spinner: ProgressBar = ProgressBar::new_spinner()
        .with_message(msg);
    spinner.enable_steady_tick(1000u64);
    spinner
}