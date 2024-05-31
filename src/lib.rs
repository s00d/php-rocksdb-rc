use ext_php_rs::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use rust_rocksdb::{DB, Options};

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
            },
            None => DB::open(&opts, &path),
        };

        match db {
            Ok(db) => Ok(RocksDB { db: Arc::new(db) }),
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn put(&self, key: String, value: String) -> PhpResult<()> {
        self.db.put(key.as_bytes(), value.as_bytes()).map_err(|e| e.to_string().into())
    }

    pub fn put_cf(&self, key: String, value: String, cf_name: String) -> PhpResult<()> {
        let cf = self.db.cf_handle(&cf_name).ok_or("Column family not found")?;
        self.db.put_cf(&cf, key.as_bytes(), value.as_bytes()).map_err(|e| e.to_string().into())
    }

    pub fn get(&self, key: String) -> PhpResult<Option<String>> {
        match self.db.get(key.as_bytes()) {
            Ok(Some(value)) => Ok(Some(String::from_utf8(value).unwrap())),
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn get_cf(&self, key: String, cf_name: String) -> PhpResult<Option<String>> {
        let cf = self.db.cf_handle(&cf_name).ok_or("Column family not found")?;
        match self.db.get_cf(&cf, key.as_bytes()) {
            Ok(Some(value)) => Ok(Some(String::from_utf8(value).unwrap())),
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn delete(&self, key: String) -> PhpResult<()> {
        self.db.delete(key.as_bytes()).map_err(|e| e.to_string().into())
    }

    pub fn delete_cf(&self, key: String, cf_name: String) -> PhpResult<()> {
        let cf = self.db.cf_handle(&cf_name).ok_or("Column family not found")?;
        self.db.delete_cf(&cf, key.as_bytes()).map_err(|e| e.to_string().into())
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
        self.db.create_cf(&cf_name, &opts).map_err(|e| e.to_string().into())
    }

    pub fn drop_column_family(&self, cf_name: String) -> PhpResult<()> {
        self.db.drop_cf(&cf_name).map_err(|e| e.to_string().into())
    }

    pub fn get_property(&self, property: String) -> PhpResult<Option<String>> {
        match self.db.property_value(&property) {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn get_property_cf(&self, property: String, cf_name: String) -> PhpResult<Option<String>> {
        let cf = self.db.cf_handle(&cf_name).ok_or("Column family not found")?;
        match self.db.property_value_cf(&cf, &property) {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn flush(&self) -> PhpResult<()> {
        self.db.flush().map_err(|e| e.to_string().into())
    }

    pub fn repair(path: String) -> PhpResult<()> {
        let opts = Options::default();
        DB::repair(&opts, path).map_err(|e| e.to_string().into())
    }

    pub fn close(&self) -> PhpResult<()> {
        Ok(())
    }
}


#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
