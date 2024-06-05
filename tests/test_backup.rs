use std::thread::sleep;
use std::time;

mod common;
use common::php_request;

fn setup() {
    common::setup();
    sleep(time::Duration::from_secs(1));
}

#[test]
fn test_backup() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_backup";
        $backupPath = __DIR__ . "/temp/backup1";
        $backup = new RocksDBBackup($dbPath, 3600); // 3600 seconds TTL
        $backup->init($backupPath);
        $backup->create();
        $info = $backup->info();
        var_dump($info);
        $backup = null; // Free the connection
    "#,
    );
    assert!(output.contains("backup_id"));
}

#[test]
fn test_restore_backup() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/backup2";

        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db = null; // Free the connection

        $backupPath = __DIR__ . "/temp/backup2";
        $restorePath = __DIR__ . "/temp/restoredb2";
        $backup = new RocksDBBackup($dbPath, 3600); // 3600 seconds TTL
        $backup->init($backupPath);
        $backup->create();
        $backup->restore(1, $restorePath);
        $db = new RocksDB($restorePath, 3600);
        $value = $db->get("key1");
        echo $value;
        $backup = null; // Free the connection
        $db = null; // Free the connection
    "#,
    );

    // Проверяем, что значение "value1" было восстановлено
    assert_eq!(output.trim(), "value1");
}
