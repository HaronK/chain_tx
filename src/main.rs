mod client;
mod engine;
mod transaction;

use std::fs::File;

use anyhow::{anyhow, ensure, Result};
use engine::Engine;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    ensure!(args.len() == 2, "Expected 1 input argument - CSV file name");

    let file_rdr = File::open(&args[1])
        .map_err(|err| anyhow!("Cannot read CSV file: {}. Error: {err}", args[1]))?;

    let mut engine = Engine::default();

    engine.apply_transactions(file_rdr)?;

    engine.print_summary();

    Ok(())
}
