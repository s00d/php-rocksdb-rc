use ext_php_rs::prelude::*;
use rust_rocksdb::{DB, Options, SnapshotWithThreadMode};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[php_class]
pub struct RocksDBSnapshot {
    db: Arc<DB>,
    snapshot: Mutex<Option<SnapshotWithThreadMode<'static, DB>>>,
}

#[php_impl]
impl RocksDBSnapshot {
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
            Ok(db) => Ok(RocksDBSnapshot {
                db: Arc::new(db),
                snapshot: Mutex::new(None),
            }),
            Err(e) => Err(e.to_string().into()),
        }
    }

    pub fn create(&self) -> PhpResult<()> {
        let snapshot = self.db.snapshot();
        let mut snap = self.snapshot.lock().unwrap();
        *snap = Some(unsafe {
            std::mem::transmute::<SnapshotWithThreadMode<DB>, SnapshotWithThreadMode<'static, DB>>(
                snapshot,
            )
        });
        Ok(())
    }

    pub fn release(&self) -> PhpResult<()> {
        let mut snap = self.snapshot.lock().unwrap();
        *snap = None;
        Ok(())
    }
}
