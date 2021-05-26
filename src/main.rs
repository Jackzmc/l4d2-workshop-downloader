use dialoguer::{theme::ColorfulTheme, Select};

mod workshop;
mod menu_import;

const SELECTIONS: &'static [&'static str] = &[
    "Import Workshop VPKs",
    "Update existing VPKs",
    "Search for new item",
    "Manage Existing Items"
];


fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {    
        match Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick a option")
            .default(0)
            .items(&SELECTIONS)
            .interact()
            .unwrap() 
        {
            0 => menu_import::handler()?,
            _ => println!("Option not implemented.")
        }
    }
}
