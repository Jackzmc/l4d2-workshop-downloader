use dialoguer::{theme::ColorfulTheme, Select, Input};

mod workshop;
mod menu_import;
mod menu_update;
mod util;
mod meta;

const SELECTIONS: &'static [&'static str] = &[
    "Import Workshop VPKs",
    "Update existing VPKs",
    "Search for new item",
    "Manage Existing Items"
];

const INITIAL_SETUP_OPTIONS: &'static [&'static str] = &[
    "Use Current Directory",
    "Input a path",
];

//#[tokio::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match meta::get_config() {
        Some(config) => config,
        None => {
            let directory = prompt_for_path();
            let config = meta::Config {
                gamedir: directory.into_os_string().into_string().unwrap(),
                downloads: Vec::new()
            };
            std::fs::write("downloader_meta.json", serde_json::to_string(&config)?)?;
            config
        }
    };

    loop {    
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
        println!("")
    }
}

fn prompt_for_path() -> std::path::PathBuf {
    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Initial Setup - Set L4D2 Addons Folder")
        .items(&INITIAL_SETUP_OPTIONS)
        .default(0)
        .interact()
        .unwrap()
    {
        0 => std::env::current_dir().unwrap(),
        1 => {
            let folder: String = Input::new()
                .with_prompt("Enter a folder location")
                .interact_text().unwrap();
            std::path::PathBuf::from(folder)

            //TODO: Verify path
        }
        _ => panic!("Item is not valid")
    }
}