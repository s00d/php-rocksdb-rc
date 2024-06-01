use ext_php_rs::prelude::*;
use rust_rocksdb::{DB, Options, Transaction, TransactionDB, TransactionDBOptions, TransactionOptions, WriteOptions};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[php_class]
pub struct RocksDBTransaction {
    db: Arc<DB>,
    transaction_db: Mutex<Option<Arc<TransactionDB>>>,
    transaction: Mutex<Option<Transaction<'static, TransactionDB>>>,
}

#[php_impl]
impl RocksDBTransaction {
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
            Ok(db) => Ok(RocksDBTransaction {
                db: Arc::new(db),
                transaction_db: Mutex::new(None),
                transaction: Mutex::new(None),
            }),
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn start(&self) -> PhpResult<()> {
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

    pub fn commit(&self) -> PhpResult<()> {
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

    pub fn rollback(&self) -> PhpResult<()> {
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

    pub fn put(
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

    pub fn get(
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

    pub fn delete(&self, key: String, cf_name: Option<String>) -> PhpResult<()> {
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

    pub fn merge(
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
