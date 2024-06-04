#![cfg_attr(all(windows, target_arch = "x86_64"), feature(abi_vectorcall))]

use ext_php_rs::prelude::*;
use rust_rocksdb::{Options, WriteBatchWithTransaction, DB};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[php_class]
pub struct RocksDBWriteBatch {
    db: Arc<DB>,
    write_batch: Mutex<Option<WriteBatchWithTransaction<false>>>,
}

#[php_impl]
impl RocksDBWriteBatch {
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
            Ok(db) => Ok(RocksDBWriteBatch {
                db: Arc::new(db),
                write_batch: Mutex::new(None),
            }),
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn start(&self) -> PhpResult<()> {
        let mut batch = self.write_batch.lock().unwrap();
        *batch = Some(WriteBatchWithTransaction::<false>::default());
        Ok(())
    }

    pub fn put(&self, key: String, value: String, cf_name: Option<String>) -> PhpResult<()> {
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref mut wb) = *batch {
            match cf_name {
                Some(cf_name) => {
                    let cf = self
                        .db
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    wb.put_cf(&cf, key.as_bytes(), value.as_bytes());
                }
                None => {
                    wb.put(key.as_bytes(), value.as_bytes());
                }
            }
        } else {
            return Err("WriteBatch not initialized".into());
        }
        Ok(())
    }

    pub fn merge(&self, key: String, value: String, cf_name: Option<String>) -> PhpResult<()> {
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref mut wb) = *batch {
            match cf_name {
                Some(cf_name) => {
                    let cf = self
                        .db
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    wb.merge_cf(&cf, key.as_bytes(), value.as_bytes());
                }
                None => {
                    wb.merge(key.as_bytes(), value.as_bytes());
                }
            }
        } else {
            return Err("WriteBatch not initialized".into());
        }
        Ok(())
    }

    pub fn delete(&self, key: String, cf_name: Option<String>) -> PhpResult<()> {
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref mut wb) = *batch {
            match cf_name {
                Some(cf_name) => {
                    let cf = self
                        .db
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    wb.delete_cf(&cf, key.as_bytes());
                }
                None => {
                    wb.delete(key.as_bytes());
                }
            }
        } else {
            return Err("WriteBatch not initialized".into());
        }
        Ok(())
    }

    pub fn write(&self) -> PhpResult<()> {
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(wb) = batch.take() {
            self.db
                .write(wb)
                .map_err(|e| PhpException::from(e.to_string()))?;
        } else {
            return Err("WriteBatch not initialized".into());
        }
        Ok(())
    }

    pub fn clear(&self) -> PhpResult<()> {
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref mut wb) = *batch {
            wb.clear();
        } else {
            return Err("WriteBatch not initialized".into());
        }
        Ok(())
    }

    pub fn destroy(&self) -> PhpResult<()> {
        let mut batch = self.write_batch.lock().unwrap();
        *batch = None;
        Ok(())
    }
}
