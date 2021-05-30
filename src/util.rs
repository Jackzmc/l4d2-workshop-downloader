use indicatif::ProgressBar;
use std::{borrow::Cow};

pub struct MenuResult {
}

pub fn setup_spinner(msg: impl Into<Cow<'static, str>>) -> ProgressBar {
    let spinner: ProgressBar = ProgressBar::new_spinner()
        .with_message(msg);
    spinner.enable_steady_tick(1000u64);
    spinner
}