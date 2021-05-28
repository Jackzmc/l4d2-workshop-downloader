use dialoguer::{theme::ColorfulTheme, Select, Input};
use console::style;
use std::path::PathBuf;

mod menu_import;
mod menu_update;
mod menu_search;
mod menu_manage;
mod util;
mod meta;

//#[tokio::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} v{}", style("L4D2 Workshop Downloader").bold(), env!("CARGO_PKG_VERSION"));
    //Grab the config or start initial setup
    let workshop = steamwebapi::Workshop::new(None);
    //TODO: Add option to save file name 
    let config = 
        if let Some(config) = meta::get_config() {
            println!("{} \"{}\"", style("Using saved directory:").bold(), &config.get_game_path_str().expect("< no path >"));
            config
        }else {
            let path: PathBuf = prompt_for_path();
            let mut config = meta::Config {
                gamedir: path,
                apikey: None,
                downloads: Vec::new(),
                include_name: true
            };
            if let Some(prompt_res) = prompt_for_apikey() {
                config.apikey = prompt_res.apikey;
            }
            
            std::fs::write("downloader_meta.json", serde_json::to_string(&config)?)?;
            config
        };

    loop {    
        println!();
        match Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick a option")
            .items(&[
                "Import Workshop VPKs",
                "Update existing VPKs",
                "Search for new item",
                "Manage Existing Items"
            ])
            .interact()
            .unwrap() 
        {
            0 => menu_import::handler(&config, &workshop)?,
            1 => menu_update::handler(&config, &workshop)?,
            2 => menu_search::handler(&config, &workshop)?,
            3 => menu_manage::handler(&config, &workshop)?,
            _ => println!("Option not implemented.")
        }
    }
}

fn prompt_for_path() -> PathBuf {
    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Initial Setup - Set L4D2 Addons Folder")
        .items(&[
            "Use Current Directory",
            "Choose a directory",
        ])
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

struct ApiKey {
    apikey: Option<String>,
}

fn prompt_for_apikey() -> Option<ApiKey> {
    println!("A Steam Web API Key is required for some functionality. Get an apikey from https://steamcommunity.com/dev/apikey.");
    println!("Leave blank to disable options");
    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a choice")
        .items(&[
            "Enter an Steam Web API Key",
            "Use Proxy - https://jackz.me/l4d2/search_public.php",
            "Do not use an apikey (disables some options)"
        ])
        .interact()
        .unwrap() 
    {
        0 => {
            let res = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter a steam web api key")
            .interact_text()
            .unwrap();
            Some(ApiKey {
                apikey: Some(res),
            })
        },
        1 => {
            Some(ApiKey {
                apikey: None,
            })
        },
        2 => None,
        _ => panic!("Unreachable")
    }
    

}