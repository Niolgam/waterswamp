use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref STATE_CODE_REGEX: Regex = Regex::new(r"^[A-Z]{2}$").unwrap();
}
