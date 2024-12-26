use std::sync::Arc;

use crate::args::ArgsCli;

pub struct Config {
    pub token: String,
    pub allowed_user_ids: Arc<Vec<u64>>,
    pub db_file_path: String,
    pub tz: String,
}

impl Config {
    pub fn new(args: ArgsCli) -> Self {
        Self {
            token: args.token,
            allowed_user_ids: Arc::new(args.allowed_user_ids),
            db_file_path: args.db_file_path,
            tz: args.tz,
        }
    }
}
