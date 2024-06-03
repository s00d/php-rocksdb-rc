
mod backup;
mod write_batch;
mod snapshot;
mod transaction;

use ext_php_rs::prelude::*;
use rust_rocksdb::{Options, DB};
use std::collections::HashMap;
use std::time::Duration;
use ext_php_rs::convert::IntoZval;
use ext_php_rs::types::{ZendHashTable, Zval};
use ext_php_rs::error::Error;

use crate::backup::RocksDBBackup;
use crate::write_batch::RocksDBWriteBatch;
use crate::snapshot::RocksDBSnapshot;
use crate::transaction::RocksDBTransaction;


#[derive(Debug)]
pub struct KeyValueResult {
    pub key: Option<String>,
    pub value: Option<String>,
}

impl IntoZval for KeyValueResult {
    const TYPE: ext_php_rs::flags::DataType = ext_php_rs::flags::DataType::Array;

    fn set_zval(self, zv: &mut Zval, _persistent: bool) -> Result<(), Error> {
        let mut ht = ZendHashTable::new();
        ht.insert("key", self.key.into_zval(false)?)?;
        ht.insert("value", self.value.into_zval(false)?)?;
        zv.set_hashtable(ht);
        Ok(())
    }
}


#[php_class]
pub struct RocksDB {
    pub db: DB,
    position: Option<Vec<u8>>,
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
                Ok(RocksDB {
                    db,
                    position: None,
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

    // -- iterator
    pub fn seek_to_first(&mut self) -> PhpResult<()> {
        let mut iter = self.db.raw_iterator();
        iter.seek_to_first();
        self.position = iter.key().map(|k| k.to_vec());
        Ok(())
    }

    pub fn seek_to_last(&mut self) -> PhpResult<()> {
        let mut iter = self.db.raw_iterator();
        iter.seek_to_last();
        self.position = iter.key().map(|k| k.to_vec());
        Ok(())
    }

    pub fn seek(&mut self, key: String) -> PhpResult<()> {
        let mut iter = self.db.raw_iterator();
        iter.seek(key.as_bytes());
        self.position = iter.key().map(|k| k.to_vec());
        Ok(())
    }

    pub fn seek_for_prev(&mut self, key: String) -> PhpResult<()> {
        let mut iter = self.db.raw_iterator();
        iter.seek_for_prev(key.as_bytes());
        self.position = iter.key().map(|k| k.to_vec());
        Ok(())
    }

    pub fn valid(&self) -> PhpResult<bool> {
        let mut iter = self.db.raw_iterator();
        if let Some(pos) = &self.position {
            iter.seek(pos);
        }
        let valid = iter.valid();
        Ok(valid)
    }

    pub fn next(&mut self) -> PhpResult<KeyValueResult> {
        let mut iter = self.db.raw_iterator();
        if let Some(pos) = &self.position {
            iter.seek(pos);
        }
        if iter.valid() {
            let key = iter.key().map(|k| String::from_utf8_lossy(k).to_string());
            let value = iter.value().map(|v| String::from_utf8_lossy(v).to_string());
            iter.next();
            self.position = iter.key().map(|k| k.to_vec());
            Ok(KeyValueResult { key, value })
        } else {
            self.position = None;
            Ok(KeyValueResult { key: None, value: None })
        }
    }

    pub fn prev(&mut self) -> PhpResult<KeyValueResult> {
        let mut iter = self.db.raw_iterator();
        if let Some(pos) = &self.position {
            iter.seek(pos);
        }

        if iter.valid() {
            let key = iter.key().map(|k| String::from_utf8_lossy(k).to_string());
            let value = iter.value().map(|v| String::from_utf8_lossy(v).to_string());
            iter.prev();
            self.position = iter.key().map(|k| k.to_vec());
            Ok(KeyValueResult { key, value })
        } else {
            self.position = None;
            Ok(KeyValueResult { key: None, value: None })
        }
    }
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
