#![cfg_attr(all(windows, target_arch = "x86_64"), feature(abi_vectorcall))]

use ext_php_rs::prelude::*;
use rust_rocksdb::backup::{BackupEngine, BackupEngineOptions, RestoreOptions};
use rust_rocksdb::{Env, Options, DB};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[php_class]
pub struct RocksDBBackup {
    db: Arc<DB>,
    backup_engine: Mutex<Option<BackupEngine>>,
}

#[php_impl]
impl RocksDBBackup {
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
            Ok(db) => Ok(RocksDBBackup {
                db: Arc::new(db),
                backup_engine: Mutex::new(None),
            }),
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn init(&self, backup_path: String) -> PhpResult<()> {
        let mut backup_engine = self.backup_engine.lock().unwrap();
        let be_opts = BackupEngineOptions::new(&backup_path).map_err(|e| e.to_string())?;
        *backup_engine = Some(
            BackupEngine::open(&be_opts, &Env::new().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?,
        );
        Ok(())
    }

    pub fn create(&self) -> PhpResult<()> {
        let mut backup_engine = self.backup_engine.lock().unwrap();
        if let Some(be) = backup_engine.as_mut() {
            be.create_new_backup(&*self.db).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn info(&self) -> PhpResult<HashMap<String, i64>> {
        let backup_engine = self.backup_engine.lock().unwrap();
        if let Some(be) = &*backup_engine {
            let backup_info = be.get_backup_info();
            let mut result = HashMap::new();
            for info in backup_info {
                result.insert("backup_id".to_string(), info.backup_id as i64);
                result.insert("timestamp".to_string(), info.timestamp as i64);
                result.insert("size".to_string(), info.size as i64);
                result.insert("num_files".to_string(), info.num_files as i64);
            }
            return Ok(result);
        }
        Err("Backup engine is not initialized".into())
    }

    pub fn purge_old(&self, num_backups_to_keep: usize) -> PhpResult<()> {
        let mut backup_engine = self.backup_engine.lock().unwrap();
        if let Some(be) = backup_engine.as_mut() {
            be.purge_old_backups(num_backups_to_keep)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn restore(&self, backup_id: u32, restore_path: String) -> PhpResult<()> {
        let mut backup_engine = self.backup_engine.lock().unwrap();
        if let Some(be) = backup_engine.as_mut() {
            let opts = RestoreOptions::default();
            be.restore_from_backup(&restore_path, &restore_path, &opts, backup_id)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
