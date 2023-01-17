pub use domain::*;
use once_cell::sync::OnceCell;
use snowflake::SnowflakeIdGenerator;
use std::sync::Mutex;

mod domain;

pub(crate) fn order_item_id_generator() -> &'static Mutex<SnowflakeIdGenerator> {
    static INSTANCE: OnceCell<Mutex<SnowflakeIdGenerator>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let generator = SnowflakeIdGenerator::new(0, 1);
        Mutex::new(generator)
    })
}
