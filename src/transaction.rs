#![cfg_attr(all(windows, target_arch = "x86_64"), feature(abi_vectorcall))]

use ext_php_rs::prelude::*;
use rust_rocksdb::{
    Options, Transaction, TransactionDB, TransactionDBOptions, TransactionOptions, WriteOptions,
};
use std::sync::{Arc, Mutex};

#[php_class]
pub struct RocksDBTransaction {
    transaction_db: Arc<TransactionDB>,
    transaction: Arc<Mutex<Option<Transaction<'static, TransactionDB>>>>,
}

fn create_transaction(transaction_db: &Arc<TransactionDB>) -> Transaction<'static, TransactionDB> {
    let txn_opts = TransactionOptions::default();
    let write_opts = WriteOptions::default();
    unsafe {
        std::mem::transmute::<Transaction<TransactionDB>, Transaction<'static, TransactionDB>>(
            transaction_db.transaction_opt(&write_opts, &txn_opts),
        )
    }
}

#[php_impl]
impl RocksDBTransaction {
    #[constructor]
    pub fn __construct(path: String) -> PhpResult<Self> {
        let txn_db_opts = TransactionDBOptions::default();
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(1000);
        opts.set_log_level(rust_rocksdb::LogLevel::Warn);

        let transaction_db = TransactionDB::open(&opts, &txn_db_opts, &path)
            .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))?;

        let transaction_db = Arc::new(transaction_db);
        let transaction = create_transaction(&transaction_db);

        Ok(RocksDBTransaction {
            transaction_db: Arc::clone(&transaction_db),
            transaction: Arc::new(Mutex::new(Some(unsafe {
                std::mem::transmute::<Transaction<TransactionDB>, Transaction<'static, TransactionDB>>(
                    transaction,
                )
            }))),
        })
    }

    #[destructor]
    pub fn __destruct(&self) {
        let mut txn_guard = self.transaction.lock().unwrap();
        if let Some(txn) = txn_guard.take() {
            let _ = txn.commit(); // Ignoring any errors on destruction
        }
    }

    pub fn commit(&self) -> PhpResult<()> {
        let mut txn_guard = self.transaction.lock().unwrap();
        if let Some(txn) = txn_guard.take() {
            txn.commit()
                .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        }
        *txn_guard = Some(create_transaction(&self.transaction_db));
        Ok(())
    }

    pub fn rollback(&self) -> PhpResult<()> {
        let mut txn_guard = self.transaction.lock().unwrap();
        if let Some(txn) = txn_guard.take() {
            txn.rollback()
                .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        }
        *txn_guard = Some(create_transaction(&self.transaction_db));
        Ok(())
    }

    pub fn set_savepoint(&self) -> PhpResult<()> {
        let txn_guard = self.transaction.lock().unwrap();
        if let Some(ref txn) = *txn_guard {
            txn.set_savepoint();
        }
        Ok(())
    }

    pub fn rollback_to_savepoint(&self) -> PhpResult<()> {
        let txn_guard = self.transaction.lock().unwrap();
        if let Some(ref txn) = *txn_guard {
            txn.rollback_to_savepoint()
                .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))?;
        }
        Ok(())
    }

    pub fn put(&self, key: String, value: String, cf_name: Option<String>) -> PhpResult<()> {
        let txn_guard = self.transaction.lock().unwrap();
        if let Some(ref txn) = *txn_guard {
            match cf_name {
                Some(cf_name) => {
                    let cf = self
                        .transaction_db
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    txn.put_cf(&cf, key.as_bytes(), value.as_bytes())
                        .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))
                }
                None => txn
                    .put(key.as_bytes(), value.as_bytes())
                    .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string())),
            }
        } else {
            Err(ext_php_rs::exception::PhpException::from(
                "No active transaction".to_string(),
            ))
        }
    }

    pub fn get(&self, key: String, cf_name: Option<String>) -> PhpResult<Option<String>> {
        let txn_guard = self.transaction.lock().unwrap();
        if let Some(ref txn) = *txn_guard {
            match cf_name {
                Some(cf_name) => {
                    let cf = self
                        .transaction_db
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    match txn.get_cf(&cf, key.as_bytes()) {
                        Ok(Some(value)) => Ok(Some(String::from_utf8(value).map_err(|e| {
                            ext_php_rs::exception::PhpException::from(e.to_string())
                        })?)),
                        Ok(None) => Ok(None),
                        Err(e) => Err(ext_php_rs::exception::PhpException::from(e.to_string())),
                    }
                }
                None => {
                    match txn.get(key.as_bytes()) {
                        Ok(Some(value)) => Ok(Some(String::from_utf8(value).map_err(|e| {
                            ext_php_rs::exception::PhpException::from(e.to_string())
                        })?)),
                        Ok(None) => Ok(None),
                        Err(e) => Err(ext_php_rs::exception::PhpException::from(e.to_string())),
                    }
                }
            }
        } else {
            Err(ext_php_rs::exception::PhpException::from(
                "No active transaction".to_string(),
            ))
        }
    }

    pub fn delete(&self, key: String, cf_name: Option<String>) -> PhpResult<()> {
        let txn_guard = self.transaction.lock().unwrap();
        if let Some(ref txn) = *txn_guard {
            match cf_name {
                Some(cf_name) => {
                    let cf = self
                        .transaction_db
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    txn.delete_cf(&cf, key.as_bytes())
                        .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))
                }
                None => txn
                    .delete(key.as_bytes())
                    .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string())),
            }
        } else {
            Err(ext_php_rs::exception::PhpException::from(
                "No active transaction".to_string(),
            ))
        }
    }

    pub fn merge(&self, key: String, value: String, cf_name: Option<String>) -> PhpResult<()> {
        let txn_guard = self.transaction.lock().unwrap();
        if let Some(ref txn) = *txn_guard {
            match cf_name {
                Some(cf_name) => {
                    let cf = self
                        .transaction_db
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    txn.merge_cf(&cf, key.as_bytes(), value.as_bytes())
                        .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string()))
                }
                None => txn
                    .merge(key.as_bytes(), value.as_bytes())
                    .map_err(|e| ext_php_rs::exception::PhpException::from(e.to_string())),
            }
        } else {
            Err(ext_php_rs::exception::PhpException::from(
                "No active transaction".to_string(),
            ))
        }
    }
}
