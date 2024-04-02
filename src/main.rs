mod client;
mod engine;
mod transaction;

use anyhow::{ensure, Result};
use engine::Engine;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    ensure!(args.len() == 2, "Expected 1 input argument - CSV file name");

    let mut engine = Engine::default();

    engine.apply_transactions(&args[1])?;

    engine.print_summary();

    Ok(())
}
