use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
pub struct ArgsCli {
    #[arg(short, long, required = true, help = "Telegram API token")]
    pub token: String,

    #[arg(short, long, required = true, help = "Allowed User IDs to use bot")]
    pub allowed_user_ids: Vec<u64>,
}
