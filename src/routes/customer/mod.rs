use std::sync::Mutex;

use once_cell::sync::OnceCell;
use snowflake::SnowflakeIdGenerator;

pub use domain::*;
pub use route::{create_customer_handler, delete_customer_handler, update_customer_handler};

mod domain;
mod route;

pub(crate) fn customer_id_generator() -> &'static Mutex<SnowflakeIdGenerator> {
    static INSTANCE: OnceCell<Mutex<SnowflakeIdGenerator>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let generator = SnowflakeIdGenerator::new(0, 1);
        Mutex::new(generator)
    })
}
