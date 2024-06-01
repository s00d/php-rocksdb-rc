use ext_php_rs::prelude::*;
use rust_rocksdb::{DB, IteratorMode, Direction, Options};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;

#[php_class]
pub struct RocksDBIterator {
    db: Arc<DB>,
    iter_position: Mutex<Option<Vec<u8>>>,
    iter_cf: Mutex<Option<String>>,
}

#[php_impl]
impl RocksDBIterator {
    #[constructor]
    pub fn __construct(path: String, ttl_secs: Option<u64>) -> PhpResult<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(1000);
        opts.set_log_level(rust_rocksdb::LogLevel::Warn);

        let db = match ttl_secs {
            Some(ttl) => {
                let duration = Duration::from_secs(ttl);
                DB::open_with_ttl(&opts, &path, duration)
            }
            None => DB::open(&opts, &path),
        };

        match db {
            Ok(db) => Ok(RocksDBIterator {
                db: Arc::new(db),
                iter_position: Mutex::new(None),
                iter_cf: Mutex::new(None),
            }),
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn iterator(&self, cf_name: Option<String>) -> PhpResult<()> {
        let mut iter_position = self.iter_position.lock().unwrap();
        *iter_position = None;
        let mut iter_cf = self.iter_cf.lock().unwrap();
        *iter_cf = cf_name;
        Ok(())
    }

    pub fn next(&self, batch_size: usize) -> PhpResult<HashMap<String, String>> {
        let mut result = HashMap::new();
        let mut iter_position = self.iter_position.lock().unwrap();
        let iter_cf = self.iter_cf.lock().unwrap();

        let iter = match &*iter_cf {
            Some(cf_name) => {
                let cf = self.db.cf_handle(cf_name).ok_or("Column family not found")?;
                if let Some(ref position) = *iter_position {
                    self.db
                        .iterator_cf(&cf, IteratorMode::From(position, Direction::Forward))
                } else {
                    self.db.iterator_cf(&cf, IteratorMode::Start)
                }
            }
            None => {
                if let Some(ref position) = *iter_position {
                    self.db
                        .iterator(IteratorMode::From(position, Direction::Forward))
                } else {
                    self.db.iterator(IteratorMode::Start)
                }
            }
        };

        for (i, item) in iter.take(batch_size).enumerate() {
            let (key, value) = item.map_err(|e| e.to_string())?;
            let key_str = String::from_utf8(key.to_vec()).map_err(|e| e.to_string())?;
            let value_str = String::from_utf8(value.to_vec()).map_err(|e| e.to_string())?;
            result.insert(key_str.clone(), value_str);

            if i == batch_size - 1 {
                *iter_position = Some(key.to_vec());
            }
        }

        Ok(result)
    }

    pub fn reset(&self) -> PhpResult<()> {
        let mut iter_position = self.iter_position.lock().unwrap();
        *iter_position = None;
        Ok(())
    }
}
