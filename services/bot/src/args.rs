use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
pub struct ArgsCli {
    #[arg(short = 't', required = true, help = "Telegram API token")]
    pub token: String,

    #[arg(short = 'u', required = true, help = "Allowed User IDs to use bot")]
    pub allowed_user_ids: Vec<u64>,

    #[arg(short = 'd', required = true, help = "DB file path")]
    pub db_file_path: String,

    #[arg(short = 'z', default_value = "Europe/Moscow", help = "Timezone")]
    pub tz: String,

    #[arg(short = 'b', action, help = "Debug mode")]
    pub debug: bool,
}
