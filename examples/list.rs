#![feature(async_closure)]

use anyhow::Result;

use stagebridge::midi::Midi;
use stagebridge::midi::device::Raw;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    Midi::<Raw>::list()?;
    Ok(())
}
