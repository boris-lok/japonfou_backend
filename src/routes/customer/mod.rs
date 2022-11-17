mod create;
mod domain;

pub use create::create_customer_handler;
pub use domain::*;
use once_cell::sync::OnceCell;
use snowflake::SnowflakeIdGenerator;
use std::sync::Mutex;

pub(crate) fn customer_id_generator() -> &'static Mutex<SnowflakeIdGenerator> {
    static INSTANCE: OnceCell<Mutex<SnowflakeIdGenerator>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let generator = SnowflakeIdGenerator::new(0, 1);
        Mutex::new(generator)
    })
}
