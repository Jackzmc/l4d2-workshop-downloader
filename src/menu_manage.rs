use crate::meta;
use crate::util;

use steamwebapi::Workshop;

pub fn handler(_config: &meta::Config, _workshop: &Workshop) -> Result<Option<util::MenuResult>, Box<dyn std::error::Error>> {
    Ok(None)
}