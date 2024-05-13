use lazy_static::lazy_static;
use sqlx::migrate::Migrator;
use std::env;

pub const DEFAULT_GAS_LIMIT: i64 = 21000;
pub const LAST_LEGACY_BLOCK_TIMESTAMP: i64 = 1713557133;
pub const LAST_LEGACY_BLOCK_NUMBER: i64 = 83999;
pub const CHAIN_ID: i64 = 178;
pub static MIGRATOR: Migrator = sqlx::migrate!();

macro_rules! account_id {
    ($last_byte:expr) => {{
        let mut array = [0u8; 20];
        array[19] = $last_byte;
        array
    }};
}

pub enum Env {
    Production,
    Development,
}

lazy_static! {
    pub static ref ENV: Env = if env::var("ENV").unwrap_or("".to_string()) == "production" {
        Env::Production
    } else {
        Env::Development
    };
    pub static ref PORT: u16 = env::var("PORT")
        .and_then(|port| Ok(port.parse().unwrap_or(3000)))
        .unwrap();
    pub static ref LETS_ENCRYPT_EMAILS: Vec<String> = env::var("LETS_ENCRYPT_EMAILS")
        .and_then(|emails| Ok(emails
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .clone()))
        .unwrap_or(vec![]);
    pub static ref LETS_ENCRYPT_DOMAINS: Vec<String> = env::var("LETS_ENCRYPT_DOMAINS")
        .and_then(|emails| Ok(emails
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .clone()))
        .unwrap_or(vec![]);
}

const _LEGACY_ACCOUNT: [u8; 20] = account_id!(0x00);
