mod menu_import;
mod menu_update;
mod menu_search;
mod menu_manage;
mod util;
mod meta;
mod logger;

use dialoguer::{theme::ColorfulTheme, Select, Input};
use console::style;
use std::path::PathBuf;
use clap::{AppSettings, Clap};
use logger::LogLevel;

//TODO: Setup file logger

#[derive(Clap)]
#[clap(version = "1.0", author = "Jackz <me@jackz.me>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short, long)]
    menu: Option<String>,
    // #[clap(short, long, parse(from_occurrences))]
    // verbose: i32,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    println!("{} v{}", style("L4D2 Workshop Downloader").bold(), env!("CARGO_PKG_VERSION"));
    //Grab the config or start initial setup
    let workshop = steam_workshop_api::Workshop::new(None);
    let logger = logger::Logger::new(PathBuf::from(std::env::current_dir().unwrap()).join("downloader.log"));
    //TODO: Add option to save file name 
    let mut config = 
        if let Some(config) = meta::Config::load() {
            if !&config.gamedir.exists() {
                eprintln!("Saved game directory does not exist: {}", &config.get_game_path_str().expect("< no path >"));
                std::process::exit(1);
            }
            println!("{} \"{}\"", style("Using saved directory:").bold(), &config.get_game_path_str().expect("< no path >"));
            config
        }else {
            let path: PathBuf = prompt_for_path();
            let config = meta::Config::new(path);
            /*if let Some(prompt_res) = prompt_for_apikey() {
                config.apikey = prompt_res.apikey;
            }*/
            if let Err(err) = config.save() {
                logger.log(LogLevel::ERROR, &format!("Failed to save configuration: {}", err));
                eprintln!("Failed to save configuration: {}", err);
                std::process::exit(1);
            }
            logger.log(LogLevel::INFO, "Saved initial config");
            config
        };

    let mut params = util::MenuParams {
        config: &mut config,
        workshop: &workshop,
        logger: &logger
    };
    //TODO: Add arg shortcut to this:
    if let Some(option) = opts.menu {
        let menu = match option.as_str() {
            "view"     | "v" | "1" | "addons" => 1,
            "search"   | "s" | "2" => 2,
            "import"   | "i" | "3" => 3,
            "update"   | "u" | "4" => 4,
            "settings" | "c" | "5" => 5,
            _ => { println!("Unknown menu provided: \"{}\"", option); 0 }
        };
        if menu > 0 {
            println!();
            logger.log(LogLevel::INFO, &format!("Flag --menu {} specified, opening menu id {}", option, menu - 1));
            open_menu(&mut params, menu - 1);
        }
    }

    loop {    
        println!();

        let res: usize = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick a option (use arrow keys to select, enter to confirm)")
            .items(&[
                "1. View Installed Addons",
                "2. Search / Download Addons",
                "3. Import Workshop Addons",
                "4. Update Existing Addons",
                "5. Change Settings",
                "Exit"
            ])
            .default(0)
            .interact()
            .unwrap();
        println!();
        logger.log(LogLevel::INFO, &format!("Opening menu id {}", res));
        open_menu(&mut params, res);

    }
}

fn open_menu(params: &mut util::MenuParams, number: usize) {
    let result = match number {
        0 => menu_manage::handler(params),
        1 => menu_search::handler(params),
        2 => menu_import::handler(params),
        3 => menu_update::handler(params),
        _ => std::process::exit(0)
    };
    match result {
        Ok(_result) => {

        },
        Err(err) => {
            eprintln!("{} {}", style("Menu returned an error:").bold(), err);
        }
    }
}

fn prompt_for_path() -> PathBuf {
    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Initial Setup - Set L4D2 Addons Folder")
        .items(&[
            "Use Current Directory",
            "Choose a directory",
            "Input a path manually"
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
                    let path = PathBuf::from(file_path)
                    .parent()
                    .unwrap()
                    .join("left4dead2")
                    .join("addons");
                    if !path.exists() {
                        println!("A valid directory was not specified. Exiting.");
                        std::process::exit(1);
                    }
                    path
                },
                _ => {
                    println!("A valid directory was not specified. Exiting.");
                    std::process::exit(1);
                }    
            }
        },
        2 => {
            match Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter a path")
                .interact_on(&console::Term::stdout())
            {
                Ok(path) => {
                    let path = PathBuf::from(path);
                    if !path.exists() {
                        println!("A valid directory was not specified. Exiting.");
                        std::process::exit(1);
                    }
                    path
                },
                Err(e) => {
                    eprintln!("An error occurred: {}", e);
                    std::process::exit(1);
                }
            }
        },
        _ => panic!("Item is not valid")
    }
}