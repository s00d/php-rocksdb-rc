mod iterator;
mod backup;
mod write_batch;
mod snapshot;
mod transaction;

use ext_php_rs::prelude::*;
use rust_rocksdb::{Options, DB};
use std::collections::HashMap;
use std::sync::{Arc};
use std::time::Duration;

use crate::iterator::RocksDBIterator;
use crate::backup::RocksDBBackup;
use crate::write_batch::RocksDBWriteBatch;
use crate::snapshot::RocksDBSnapshot;
use crate::transaction::RocksDBTransaction;


#[php_class]
pub struct RocksDB {
    db: Arc<DB>,
}

#[php_impl]
impl RocksDB {
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
            Ok(db) => {
                let db_arc = Arc::new(db);
                Ok(RocksDB {
                    db: db_arc.clone(),
                })
            }
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn put(&self, key: String, value: String, cf_name: Option<String>) -> PhpResult<()> {
        match cf_name {
            Some(cf_name) => {
                let cf = self
                    .db
                    .cf_handle(&cf_name)
                    .ok_or("Column family not found")?;
                self.db
                    .put_cf(&cf, key.as_bytes(), value.as_bytes())
                    .map_err(|e| e.to_string().into())
            }
            None => self
                .db
                .put(key.as_bytes(), value.as_bytes())
                .map_err(|e| e.to_string().into()),
        }
    }

    pub fn get(&self, key: String, cf_name: Option<String>) -> PhpResult<Option<String>> {
        match cf_name {
            Some(cf_name) => {
                let cf = self
                    .db
                    .cf_handle(&cf_name)
                    .ok_or("Column family not found")?;
                match self.db.get_cf(&cf, key.as_bytes()) {
                    Ok(Some(value)) => {
                        Ok(Some(String::from_utf8(value).map_err(|e| e.to_string())?))
                    }
                    Ok(None) => Ok(None),
                    Err(e) => Err(e.to_string().into()),
                }
            }
            None => match self.db.get(key.as_bytes()) {
                Ok(Some(value)) => Ok(Some(String::from_utf8(value).map_err(|e| e.to_string())?)),
                Ok(None) => Ok(None),
                Err(e) => Err(e.to_string().into()),
            },
        }
    }

    pub fn merge(&self, key: String, value: String, cf_name: Option<String>) -> PhpResult<()> {
        match cf_name {
            Some(cf_name) => {
                let cf = self
                    .db
                    .cf_handle(&cf_name)
                    .ok_or("Column family not found")?;
                self.db
                    .merge_cf(&cf, key.as_bytes(), value.as_bytes())
                    .map_err(|e| e.to_string().into())
            }
            None => self
                .db
                .merge(key.as_bytes(), value.as_bytes())
                .map_err(|e| e.to_string().into()),
        }
    }

    pub fn delete(&self, key: String, cf_name: Option<String>) -> PhpResult<()> {
        match cf_name {
            Some(cf_name) => {
                let cf = self
                    .db
                    .cf_handle(&cf_name)
                    .ok_or("Column family not found")?;
                self.db
                    .delete_cf(&cf, key.as_bytes())
                    .map_err(|e| e.to_string().into())
            }
            None => self
                .db
                .delete(key.as_bytes())
                .map_err(|e| e.to_string().into()),
        }
    }

    pub fn list_column_families(path: String) -> PhpResult<Vec<String>> {
        let opts = Options::default();
        match DB::list_cf(&opts, path) {
            Ok(cfs) => Ok(cfs),
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn create_column_family(&self, cf_name: String) -> PhpResult<()> {
        let opts = Options::default();
        self.db
            .create_cf(&cf_name, &opts)
            .map_err(|e| e.to_string().into())
    }

    pub fn drop_column_family(&self, cf_name: String) -> PhpResult<()> {
        self.db.drop_cf(&cf_name).map_err(|e| e.to_string().into())
    }

    pub fn get_property(
        &self,
        property: String,
        cf_name: Option<String>,
    ) -> PhpResult<Option<String>> {
        match cf_name {
            Some(cf_name) => {
                let cf = self
                    .db
                    .cf_handle(&cf_name)
                    .ok_or("Column family not found")?;
                match self.db.property_value_cf(&cf, &property) {
                    Ok(Some(value)) => Ok(Some(value)),
                    Ok(None) => Ok(None),
                    Err(e) => Err(e.to_string().into()),
                }
            }
            None => match self.db.property_value(&property) {
                Ok(Some(value)) => Ok(Some(value)),
                Ok(None) => Ok(None),
                Err(e) => Err(e.to_string().into()),
            },
        }
    }

    pub fn flush(&self, cf_name: Option<String>) -> PhpResult<()> {
        match cf_name {
            Some(cf_name) => {
                let cf = self
                    .db
                    .cf_handle(&cf_name)
                    .ok_or("Column family not found")?;
                self.db.flush_cf(&cf).map_err(|e| e.to_string().into())
            }
            None => self.db.flush().map_err(|e| e.to_string().into()),
        }
    }

    pub fn repair(path: String) -> PhpResult<()> {
        let opts = Options::default();
        DB::repair(&opts, path).map_err(|e| e.to_string().into())
    }

    pub fn close(&self) -> PhpResult<()> {
        Ok(())
    }

    pub fn all(&self, cf_name: Option<String>) -> PhpResult<HashMap<String, String>> {
        let mut result = HashMap::new();
        let mut iter = match cf_name {
            Some(cf_name) => {
                let cf = self
                    .db
                    .cf_handle(&cf_name)
                    .ok_or("Column family not found")?;
                self.db.iterator_cf(&cf, rust_rocksdb::IteratorMode::Start)
            }
            None => self.db.iterator(rust_rocksdb::IteratorMode::Start),
        };

        while let Some(Ok((key, value))) = iter.next() {
            let key_str = String::from_utf8(key.to_vec()).map_err(|e| e.to_string())?;
            let value_str = String::from_utf8(value.to_vec()).map_err(|e| e.to_string())?;
            result.insert(key_str, value_str);
        }

        Ok(result)
    }

    pub fn keys(&self, cf_name: Option<String>) -> PhpResult<Vec<String>> {
        let all_data = self.all(cf_name)?;
        Ok(all_data.keys().cloned().collect())
    }
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
