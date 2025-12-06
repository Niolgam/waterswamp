use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref STATE_CODE_REGEX: Regex = Regex::new(r"^[A-Z]{2}$").unwrap();
    pub static ref HEX_COLOR_REGEX: Regex = Regex::new(r"^#[0-9A-Fa-f]{6}$").unwrap();
}
