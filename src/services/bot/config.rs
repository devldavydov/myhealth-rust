pub struct Config {
    token: String,
    allowed_user_ids: Vec<i64>
}

impl Config {
    pub fn new(token: String, allowed_user_ids: Vec<i64>) -> Config {
        Config {
            token,
            allowed_user_ids,
        }
    }
}