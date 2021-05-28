use crate::meta;

use steamwebapi::Workshop;

pub fn handler(_config: &meta::Config, _workshop: &Workshop) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}