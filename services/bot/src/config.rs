use std::sync::Arc;

use crate::args::ArgsCli;

pub struct Config {
    pub token: String,
    pub allowed_user_ids: Arc<Vec<u64>>,
    pub tz: String,
}

impl Config {
    pub fn new(args: ArgsCli) -> Self {
        Self {
            token: args.token,
            allowed_user_ids: Arc::new(args.allowed_user_ids),
            tz: args.tz,
        }
    }
}
