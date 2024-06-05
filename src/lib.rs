#![cfg_attr(all(windows, target_arch = "x86_64"), feature(abi_vectorcall))]

mod backup;
mod transaction;
mod write_batch;

use ext_php_rs::convert::IntoZval;
use ext_php_rs::error::Error;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ZendHashTable, Zval};
use json_patch::Patch;
use rust_rocksdb::{
    ColumnFamilyDescriptor, DBWithThreadMode, MergeOperands, Options, SingleThreaded, DB,
};
use serde_json::{from_value, Value};
use std::collections::HashMap;
use std::time::Duration;

use crate::backup::RocksDBBackup;
use crate::transaction::RocksDBTransaction;
use crate::write_batch::RocksDBWriteBatch;

fn json_merge(
    _new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
    // Decode the existing value
    let mut doc: Value = if let Some(val) = existing_val {
        serde_json::from_slice(val).unwrap_or(Value::Array(vec![]))
    } else {
        Value::Array(vec![])
    };

    // Process each operand
    for op in operands {
        if let Ok(patch) = serde_json::from_slice::<Value>(op) {
            let p: Patch = from_value(patch).unwrap();
            json_patch::patch(&mut doc, &p).unwrap();
        }
    }

    // Serialize the updated JSON object back to bytes
    Some(serde_json::to_vec(&doc).unwrap())
}

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
    pub db: DBWithThreadMode<SingleThreaded>,
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
        opts.set_compression_type(rust_rocksdb::DBCompressionType::Snappy);
        opts.set_merge_operator_associative("json_merge", json_merge);

        let cf_names = DB::list_cf(&opts, &path).unwrap_or(vec!["default".to_string()]);
        let cf_descriptors: Vec<ColumnFamilyDescriptor> = cf_names
            .iter()
            .map(|name| {
                let mut cf_opts = Options::default();
                cf_opts.set_merge_operator_associative("json_merge", json_merge);
                ColumnFamilyDescriptor::new(name, cf_opts)
            })
            .collect();

        let db = match ttl_secs {
            Some(ttl) => {
                let duration = Duration::from_secs(ttl);
                DBWithThreadMode::open_cf_descriptors_with_ttl(
                    &opts,
                    &path,
                    cf_descriptors,
                    duration,
                )
            }
            None => DBWithThreadMode::open_cf_descriptors(&opts, &path, cf_descriptors),
        };

        match db {
            Ok(db) => Ok(RocksDB { db, position: None }),
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

    pub fn create_column_family(&mut self, cf_name: String) -> PhpResult<()> {
        let cf_exists = self.db.cf_handle(&cf_name).is_some();
        if cf_exists {
            return Ok(());
        }

        let mut opts = Options::default();
        opts.set_merge_operator_associative("json_merge", json_merge);
        self.db
            .create_cf(&cf_name, &opts)
            .map_err(|e| e.to_string().into())
    }

    pub fn drop_column_family(&mut self, cf_name: String) -> PhpResult<()> {
        let cf_exists = self.db.cf_handle(&cf_name).is_some();
        if !cf_exists {
            return Ok(());
        }

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
            Ok(KeyValueResult {
                key: None,
                value: None,
            })
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
            Ok(KeyValueResult {
                key: None,
                value: None,
            })
        }
    }

    pub fn compact_range(&self, start: Option<String>, end: Option<String>, cf_name: Option<String>) -> PhpResult<()> {
        match cf_name {
            Some(cf_name) => {
                let cf = self
                    .db
                    .cf_handle(&cf_name)
                    .ok_or("Column family not found")?;
                self.db
                    .compact_range_cf(&cf, start.as_ref().map(|s| s.as_bytes()), end.as_ref().map(|s| s.as_bytes()));
            }
            None => {
                self.db
                    .compact_range(start.as_ref().map(|s| s.as_bytes()), end.as_ref().map(|s| s.as_bytes()));
            }
        }
        Ok(())
    }


    pub fn get_live_files(&self) -> PhpResult<Vec<String>> {
        let live_files = self.db.live_files().map_err(|e| PhpException::from(e.to_string()))?;
        let live_file_names = live_files.iter().map(|lf| lf.name.clone()).collect();
        Ok(live_file_names)
    }

    pub fn set_options(&self, options: HashMap<String, String>, cf_name: Option<String>) -> PhpResult<()> {
        let options_vec: Vec<(&str, &str)> = options.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        match cf_name {
            Some(cf_name) => {
                let cf = self
                    .db
                    .cf_handle(&cf_name)
                    .ok_or("Column family not found")?;
                let _ = self.db.set_options_cf(cf, &options_vec);
            }
            None => {
                let _ = self.db.set_options(&options_vec);
            }
        }
        Ok(())
    }

    pub fn set_compression(&self, compression_type: String, cf_name: Option<String>) -> PhpResult<()> {
        let compression = match compression_type.as_str() {
            "none" => rust_rocksdb::DBCompressionType::None,
            "snappy" => rust_rocksdb::DBCompressionType::Snappy,
            "zlib" => rust_rocksdb::DBCompressionType::Zlib,
            "bzip2" => rust_rocksdb::DBCompressionType::Bz2,
            "lz4" => rust_rocksdb::DBCompressionType::Lz4,
            "lz4hc" => rust_rocksdb::DBCompressionType::Lz4hc,
            "zstd" => rust_rocksdb::DBCompressionType::Zstd,
            _ => return Err("Invalid compression type".into()),
        };
        let mut opts = Options::default();
        opts.set_compression_type(compression);
        match cf_name {
            Some(cf_name) => {
                let cf = self.db.cf_handle(&cf_name).ok_or("Column family not found")?;
                self.db.set_options_cf(cf, &[("compression", compression_type.as_str())])
            }
            None => self.db.set_options(&[("compression", compression_type.as_str())]),
        }.map_err(|e| e.to_string().into())
    }

    pub fn set_write_buffer_size(&self, size: usize, cf_name: Option<String>) -> PhpResult<()> {
        let mut opts = Options::default();
        opts.set_write_buffer_size(size);
        match cf_name {
            Some(cf_name) => {
                let cf = self.db.cf_handle(&cf_name).ok_or("Column family not found")?;
                self.db.set_options_cf(cf, &[("write_buffer_size", size.to_string().as_str())])
            }
            None => self.db.set_options(&[("write_buffer_size", size.to_string().as_str())]),
        }.map_err(|e| e.to_string().into())
    }

    pub fn set_cache_size(&self, size: usize, cf_name: Option<String>) -> PhpResult<()> {
        let mut opts = Options::default();
        let mut cache = rust_rocksdb::BlockBasedOptions::default();
        cache.set_block_cache(&rust_rocksdb::Cache::new_lru_cache(size));
        opts.set_block_based_table_factory(&cache);
        match cf_name {
            Some(cf_name) => {
                let cf = self.db.cf_handle(&cf_name).ok_or("Column family not found")?;
                self.db.set_options_cf(cf, &[("block_cache", size.to_string().as_str())])
            }
            None => self.db.set_options(&[("block_cache", size.to_string().as_str())]),
        }.map_err(|e| e.to_string().into())
    }
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
