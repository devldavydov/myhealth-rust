mod cli;

use anyhow::Result;

fn main() -> Result<()> {
    let mut service = cli::parse();
    service.run()
}
