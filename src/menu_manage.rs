use crate::workshop::Workshop;
use crate::util;
use crate::meta;

pub fn handler(_config: &meta::Config, workshop: &Workshop) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}