use std::time::Duration;

pub struct ExpiringValue {
    pub value: Vec<u8>,
    pub ts: Duration,
    pub expiry: Option<Duration>,
}

#[derive(Clone)]
pub struct Config {
    pub dir: String,
    pub db_file_name: String,
}
