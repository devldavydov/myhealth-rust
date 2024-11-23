use std::sync::Arc;

use bot::service::{Config, Service};
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[arg(short, long, required=true)]
    token: String,    
    #[arg(short, long, required=true)]
    allowed_user_ids: Vec<u64>,
}

fn main() -> Result<()>{
    let bot_service = Service::new(parse_config());
    bot_service.run()
}

fn parse_config() -> Config {
    let args = Args::parse();

    Config {
        token: args.token,
        allowed_user_ids: Arc::new(args.allowed_user_ids)
    }
}