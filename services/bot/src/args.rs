use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
pub struct ArgsCli {
    #[arg(short = 't', required = true, help = "Telegram API token")]
    pub token: String,

    #[arg(short = 'u', required = true, help = "Allowed User IDs to use bot")]
    pub allowed_user_ids: Vec<u64>,

    #[arg(short = 'z', default_value = "Europe/Moscow", help = "Timezone")]
    pub tz: String,
}
