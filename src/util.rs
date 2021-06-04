use indicatif::ProgressBar;
use std::{borrow::Cow};
use regex::Regex;
use lazy_static::lazy_static;

use crate::meta::Config;
use crate::logger::Logger;
use steamwebapi::Workshop;

pub struct MenuResult {

}

pub struct MenuParams<'a> {
    pub config: &'a mut Config,
    pub workshop: &'a Workshop,
    pub logger: &'a Logger
}

pub fn setup_spinner(msg: impl Into<Cow<'static, str>>) -> ProgressBar {
    let spinner: ProgressBar = ProgressBar::new_spinner()
        .with_message(msg);
    spinner.enable_steady_tick(1000u64);
    spinner
}

pub struct Regexes {}
impl Regexes {
    pub fn get_filename_addonid(filename: &str) -> Option<String>  {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r"([0-9]{7,})").unwrap();
        }
        if let Some(mat) = REGEX.find(&filename) {
            return Some(filename[mat.start()..mat.end()].to_string());
        } else {
            None
        }
    }
    pub fn get_id_from_workshop_url(url: &str) -> Option<String> {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r"https?://steamcommunity.com/(workshop|sharedfiles)/filedetails/\?id=([0-9]+)").unwrap();
        }
        //https://steamcommunity.com/sharedfiles/filedetails/?id=1558774205
        if let Some(caps) = REGEX.captures(url) {
            if caps.len() == 3 {
                return Some(caps[2].to_string())
            } 
        }
        None
    }
}