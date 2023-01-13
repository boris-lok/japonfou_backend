use std::sync::Mutex;

use once_cell::sync::OnceCell;
use snowflake::SnowflakeIdGenerator;

pub use domain::*;
pub use route::*;

mod domain;
mod route;

pub(crate) fn product_id_generator() -> &'static Mutex<SnowflakeIdGenerator> {
    static INSTANCE: OnceCell<Mutex<SnowflakeIdGenerator>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let generator = SnowflakeIdGenerator::new(0, 2);
        Mutex::new(generator)
    })
}
