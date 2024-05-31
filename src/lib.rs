use ext_php_rs::prelude::*;
use rust_rocksdb::backup::{BackupEngine, BackupEngineOptions, RestoreOptions};
use rust_rocksdb::{
    Direction, Env, IteratorMode, Options, SnapshotWithThreadMode, Transaction, TransactionDB,
    TransactionDBOptions, TransactionOptions, WriteBatchWithTransaction, WriteOptions, DB,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[php_class]
pub struct RocksDB {
    db: Arc<DB>,
    iter_position: Mutex<Option<Vec<u8>>>,
    iter_cf: Mutex<Option<String>>,
    backup_engine: Mutex<Option<BackupEngine>>,
    write_batch: Mutex<Option<WriteBatchWithTransaction<false>>>,
    snapshot: Mutex<Option<SnapshotWithThreadMode<'static, DB>>>,
    transaction_db: Mutex<Option<Arc<TransactionDB>>>,
    transaction: Mutex<Option<Transaction<'static, TransactionDB>>>,
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
            Ok(db) => Ok(RocksDB {
                db: Arc::new(db),
                iter_position: Mutex::new(None),
                iter_cf: Mutex::new(None),
                backup_engine: Mutex::new(None),
                write_batch: Mutex::new(None),
                snapshot: Mutex::new(None),
                transaction_db: Mutex::new(None),
                transaction: Mutex::new(None),
            }),
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

    // iterator
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
                let cf = self
                    .db
                    .cf_handle(cf_name)
                    .ok_or("Column family not found")?;
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

    // Backup Methods
    pub fn backup_init(&self, backup_path: String) -> PhpResult<()> {
        let mut backup_engine = self.backup_engine.lock().unwrap();
        let be_opts = BackupEngineOptions::new(&backup_path).map_err(|e| e.to_string())?;
        *backup_engine = Some(
            BackupEngine::open(&be_opts, &Env::new().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?,
        );
        Ok(())
    }

    pub fn create_backup(&self) -> PhpResult<()> {
        let mut backup_engine = self.backup_engine.lock().unwrap();
        if let Some(be) = backup_engine.as_mut() {
            be.create_new_backup(&*self.db).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn get_backup_info(&self) -> PhpResult<HashMap<String, i64>> {
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

    pub fn purge_old_backups(&self, num_backups_to_keep: usize) -> PhpResult<()> {
        let mut backup_engine = self.backup_engine.lock().unwrap();
        if let Some(be) = backup_engine.as_mut() {
            be.purge_old_backups(num_backups_to_keep)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn restore_backup(&self, backup_id: u32, restore_path: String) -> PhpResult<()> {
        let mut backup_engine = self.backup_engine.lock().unwrap();
        if let Some(be) = backup_engine.as_mut() {
            let opts = RestoreOptions::default();
            be.restore_from_backup(&restore_path, &restore_path, &opts, backup_id)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    // ---- write_batch
    pub fn start_write_batch(&self) -> PhpResult<()> {
        let mut batch = self.write_batch.lock().unwrap();
        *batch = Some(WriteBatchWithTransaction::<false>::default());
        Ok(())
    }

    pub fn put_in_batch(
        &self,
        key: String,
        value: String,
        cf_name: Option<String>,
    ) -> PhpResult<()> {
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

    pub fn merge_in_batch(
        &self,
        key: String,
        value: String,
        cf_name: Option<String>,
    ) -> PhpResult<()> {
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

    pub fn delete_in_batch(&self, key: String, cf_name: Option<String>) -> PhpResult<()> {
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

    pub fn write_batch(&self) -> PhpResult<()> {
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

    pub fn clear_batch(&self) -> PhpResult<()> {
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref mut wb) = *batch {
            wb.clear();
        } else {
            return Err("WriteBatch not initialized".into());
        }
        Ok(())
    }

    pub fn destroy_batch(&self) -> PhpResult<()> {
        let mut batch = self.write_batch.lock().unwrap();
        *batch = None;
        Ok(())
    }

    // --- snapshot
    pub fn create_snapshot(&self) -> PhpResult<()> {
        let snapshot = self.db.snapshot();
        let mut snap = self.snapshot.lock().unwrap();
        *snap = Some(unsafe {
            std::mem::transmute::<SnapshotWithThreadMode<DB>, SnapshotWithThreadMode<'static, DB>>(
                snapshot,
            )
        });
        Ok(())
    }

    pub fn release_snapshot(&self) -> PhpResult<()> {
        let mut snap = self.snapshot.lock().unwrap();
        *snap = None;
        Ok(())
    }

    // ------ transaction
    pub fn start_transaction(&self) -> PhpResult<()> {
        let txn_db_opts = TransactionDBOptions::default();
        let opts = Options::default();
        let path = self.db.path().to_str().unwrap().to_string();

        let txn_db = TransactionDB::open(&opts, &txn_db_opts, &path)
            .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))?;

        let txn_db_arc = Arc::new(txn_db);
        let txn_opts = TransactionOptions::default();
        let write_opts = WriteOptions::default();
        let transaction = txn_db_arc.transaction_opt(&write_opts, &txn_opts);

        {
            let mut txn_db_lock = self.transaction_db.lock().unwrap();
            *txn_db_lock = Some(txn_db_arc.clone());
        }

        let mut txn = self.transaction.lock().unwrap();
        *txn = Some(unsafe {
            std::mem::transmute::<Transaction<TransactionDB>, Transaction<'static, TransactionDB>>(
                transaction,
            )
        });
        Ok(())
    }

    pub fn commit_transaction(&self) -> PhpResult<()> {
        let mut txn = self.transaction.lock().unwrap();
        if let Some(transaction) = txn.take() {
            transaction
                .commit()
                .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        } else {
            return Err("Transaction not started".into());
        }
        Ok(())
    }

    pub fn rollback_transaction(&self) -> PhpResult<()> {
        let mut txn = self.transaction.lock().unwrap();
        if let Some(transaction) = txn.take() {
            transaction
                .rollback()
                .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        } else {
            return Err("Transaction not started".into());
        }
        Ok(())
    }

    pub fn set_savepoint(&self) -> PhpResult<()> {
        let txn = self.transaction.lock().unwrap();
        if let Some(ref transaction) = *txn {
            transaction.set_savepoint();
            Ok(())
        } else {
            return Err("Transaction not started".into());
        }
    }

    pub fn rollback_to_savepoint(&self) -> PhpResult<()> {
        let txn = self.transaction.lock().unwrap();
        if let Some(ref transaction) = *txn {
            transaction
                .rollback_to_savepoint()
                .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))
        } else {
            return Err("Transaction not started".into());
        }
    }

    pub fn put_in_transaction(
        &self,
        key: String,
        value: String,
        cf_name: Option<String>,
    ) -> PhpResult<()> {
        let txn = self.transaction.lock().unwrap();
        if let Some(ref transaction) = *txn {
            match cf_name {
                Some(cf_name) => {
                    let txn_db = self.transaction_db.lock().unwrap();
                    let cf = txn_db
                        .as_ref()
                        .unwrap()
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    transaction
                        .put_cf(&cf, key.as_bytes(), value.as_bytes())
                        .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))
                }
                None => transaction
                    .put(key.as_bytes(), value.as_bytes())
                    .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string())),
            }
        } else {
            return Err("Transaction not started".into());
        }
    }

    pub fn get_in_transaction(
        &self,
        key: String,
        cf_name: Option<String>,
    ) -> PhpResult<Option<String>> {
        let txn = self.transaction.lock().unwrap();
        if let Some(ref transaction) = *txn {
            match cf_name {
                Some(cf_name) => {
                    let txn_db = self.transaction_db.lock().unwrap();
                    let cf = txn_db
                        .as_ref()
                        .unwrap()
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    match transaction.get_cf(&cf, key.as_bytes()) {
                        Ok(Some(value)) => Ok(Some(String::from_utf8(value).map_err(|e| {
                            ext_php_rs::exception::PhpException::from(e.to_string())
                        })?)),
                        Ok(None) => Ok(None),
                        Err(e) => Err(ext_php_rs::exception::PhpException::from(e.to_string())),
                    }
                }
                None => {
                    match transaction.get(key.as_bytes()) {
                        Ok(Some(value)) => Ok(Some(String::from_utf8(value).map_err(|e| {
                            ext_php_rs::exception::PhpException::from(e.to_string())
                        })?)),
                        Ok(None) => Ok(None),
                        Err(e) => Err(ext_php_rs::exception::PhpException::from(e.to_string())),
                    }
                }
            }
        } else {
            return Err("Transaction not started".into());
        }
    }

    pub fn delete_in_transaction(&self, key: String, cf_name: Option<String>) -> PhpResult<()> {
        let txn = self.transaction.lock().unwrap();
        if let Some(ref transaction) = *txn {
            match cf_name {
                Some(cf_name) => {
                    let txn_db = self.transaction_db.lock().unwrap();
                    let cf = txn_db
                        .as_ref()
                        .unwrap()
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    transaction
                        .delete_cf(&cf, key.as_bytes())
                        .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))
                }
                None => transaction
                    .delete(key.as_bytes())
                    .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string())),
            }
        } else {
            return Err("Transaction not started".into());
        }
    }

    pub fn merge_in_transaction(
        &self,
        key: String,
        value: String,
        cf_name: Option<String>,
    ) -> PhpResult<()> {
        let txn = self.transaction.lock().unwrap();
        if let Some(ref transaction) = *txn {
            match cf_name {
                Some(cf_name) => {
                    let txn_db = self.transaction_db.lock().unwrap();
                    let cf = txn_db
                        .as_ref()
                        .unwrap()
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    transaction
                        .merge_cf(&cf, key.as_bytes(), value.as_bytes())
                        .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))
                }
                None => transaction
                    .merge(key.as_bytes(), value.as_bytes())
                    .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string())),
            }
        } else {
            return Err("Transaction not started".into());
        }
    }
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
