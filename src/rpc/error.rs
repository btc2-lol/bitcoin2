use lazy_static::lazy_static;
#[derive(Debug, Clone)]
pub struct Error {
    pub code: i32,
    pub message: String,
}

lazy_static! {
    pub static ref PARSE_ERROR: Error = Error {
        code: -32700,
        message: "Parse Error".to_string()
    };
    pub static ref INVALID_SENDER: Error = Error {
        code: -32000,
        message: "Invalid Sender".to_string()
    };
    pub static ref SMART_CONTACT_ERROR: i32 = -32001;
}
pub type Result<T> = std::result::Result<T, Error>;
