use std::sync::Arc;

use myhealth::services::bot::service::{Config, Service};
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

#[tokio::main]
async fn main() -> Result<()>{
    pretty_env_logger::init();
    log::info!("Starting MyHealth bot...");

    let bot_service = Service::new(parse_config());
    bot_service.run().await
}

fn parse_config() -> Config {
    let args = Args::parse();

    Config {
        token: args.token,
        allowed_user_ids: Arc::new(args.allowed_user_ids)
    }
}