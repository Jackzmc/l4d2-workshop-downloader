use dialoguer::{theme::ColorfulTheme, Select};
use console::style;
use std::path::PathBuf;

mod workshop;
mod menu_import;
mod menu_update;
mod util;
mod meta;

const SELECTIONS: &[&str] = &[
    "Import Workshop VPKs",
    "Update existing VPKs",
    "Search for new item",
    "Manage Existing Items"
];

const INITIAL_SETUP_OPTIONS: &[&str] = &[
    "Use Current Directory",
    "Choose a directory",
];

//#[tokio::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} v{}", style("L4D2 Workshop Downloader").bold(), env!("CARGO_PKG_VERSION"));
    //Grab the config or start initial setup
    let config = 
        if let Some(config) = meta::get_config() {
            println!("{} \"{}\"", style("Using saved directory:").bold(), config.get_game_path_str().expect("< no path >"));
            config
        }else {
            let path: PathBuf = prompt_for_path();
            println!("PATH SELECTED: {}", path.to_string_lossy());
            let config = meta::Config {
                gamedir: path,
                downloads: Vec::new()
            };
            std::fs::write("downloader_meta.json", serde_json::to_string(&config)?)?;
            config
        };

    loop {    
        println!();
        match Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick a option")
            .items(&SELECTIONS)
            .interact()
            .unwrap() 
        {
            0 => menu_import::handler(&config)?,
            1 => menu_update::handler(&config)?,
            _ => println!("Option not implemented.")
        }
    }
}

fn prompt_for_path() -> PathBuf {
    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Initial Setup - Set L4D2 Addons Folder")
        .items(&INITIAL_SETUP_OPTIONS)
        .default(0)
        .interact()
        .unwrap()
    {
        0 => std::env::current_dir().unwrap(),
        1 => {
            match tinyfiledialogs::open_file_dialog(
                "Choose where Left 4 Dead 2 is installed", 
                "",
                Some((&["left4dead2.exe"], "left4dead2.exe"))
            ) {
                Some(file_path) => {
                    PathBuf::from(file_path)
                    .parent()
                    .unwrap()
                    .join("left4dead2")
                    .join("addons")
                },
                _ => {
                    println!("A valid directory was not specified. Exiting.");
                    std::process::exit(1);
                }    
            }
        }
        _ => panic!("Item is not valid")
    }
}
