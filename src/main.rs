mod cli;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let mut service = cli::parse();
    service.run().context("main service run")
}
