use once_cell::sync::OnceCell;
use regex::Regex;

pub mod authentication;
pub mod configuration;
pub mod errors;
pub mod routes;
pub mod startup;
pub mod telemetry;
pub mod utils;

pub fn get_phone_number_regex() -> &'static Regex {
    static INSTANCE: OnceCell<Regex> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        Regex::new(r#"^[\+]?[(]?[0-9]{3}[)]?[-\s\.]?[0-9]{3}[-\s\.]?[0-9]{4,6}$"#).unwrap()
    })
}
