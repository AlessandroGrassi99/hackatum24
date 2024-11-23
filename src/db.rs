use rocksdb::{DBWithThreadMode, MultiThreaded, Options};
use std::sync::Arc;

pub type Database = Arc<DBWithThreadMode<MultiThreaded>>;

pub fn init_db() -> Result<Database, Box<dyn std::error::Error>> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DBWithThreadMode::<MultiThreaded>::open(&opts, "offers.db")?;
    Ok(Arc::new(db))
}