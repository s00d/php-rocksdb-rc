use indoc::indoc;
use std::thread::sleep;
use std::time;

mod common;
use common::php_request;

fn setup() {
    common::setup();
    sleep(time::Duration::from_secs(1));
}

#[test]
fn test_put() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db = null; // Free the connection

        $db = new RocksDB($dbPath, 3600);
        $value = $db->get("key1");
        var_dump($value);
        $db = null; // Free the connection
    "#});
    assert_eq!(
        indoc! {r#"
            string(6) "value1"
        "#},
        output
    );
}

#[test]
fn test_get() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db = null; // Free the connection

        $db = new RocksDB($dbPath, 3600);
        $value = $db->get("key1");
        var_dump($value);
        $db = null; // Free the connection
    "#});
    assert_eq!(
        indoc! {r#"
            string(6) "value1"
        "#},
        output
    );
}

#[test]
fn test_delete() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->delete("key1");
        $db = null; // Free the connection

        $db = new RocksDB($dbPath, 3600);
        $value = $db->get("key1");
        var_dump($value);
        $db = null; // Free the connection
    "#});
    assert_eq!(
        indoc! {r#"
            NULL
        "#},
        output
    );
}

#[test]
fn test_iterator() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_iter";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->flush();
        $db->put("key_vvv", "value_a");
        $db->put("key_ggg", "value_b");
        $db->put("key_hhh", "value_c");

        $db->seekToFirst();
        $result = [];
        while ($db->valid()) {
            $res = $db->next();
            $key = $res['key'];
            $value = $res['value'];

            $result[$key] = $value;
        }
        var_dump($result);
        $db = null; // Free the connection
    "#});

    assert_eq!(
        indoc! {r#"
            array(3) {
              ["key_ggg"]=>
              string(7) "value_b"
              ["key_hhh"]=>
              string(7) "value_c"
              ["key_vvv"]=>
              string(7) "value_a"
            }
        "#},
        output
    );
}

#[test]
fn test_backup() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb";
        $backupPath = __DIR__ . "/temp/backup";
        $backup = new RocksDBBackup($dbPath, 3600); // 3600 seconds TTL
        $backup->init($backupPath);
        $backup->create();
        $info = $backup->info();
        var_dump($info);
        $backup = null; // Free the connection
    "#});
    assert!(output.contains("backup_id"));
}

#[test]
fn test_write_batch() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb";
        $write_batch = new RocksDBWriteBatch($dbPath, 3600); // 3600 seconds TTL
        $write_batch->start();
        $write_batch->put("key1", "value1");
        $write_batch->put("key2", "value2");
        $write_batch->write();
        $write_batch = null; // Free the connection

        $db = new RocksDB($dbPath, 3600);
        $value1 = $db->get("key1");
        $value2 = $db->get("key2");
        var_dump($value1);
        var_dump($value2);
        $db = null; // Free the connection
    "#});
    assert_eq!(
        indoc! {r#"
            string(6) "value1"
            string(6) "value2"
        "#},
        output
    );
}

#[test]
fn test_commit_transaction() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_transaction";

        // Test commit
        $transaction = new RocksDBTransaction($dbPath);
        $transaction->put("key1", "value1");
        $transaction->commit();
        $transaction = null; // Free the connection

        $db = new RocksDBTransaction($dbPath);
        $value = $db->get("key1");
        var_dump($value); // Expecting value1
        $db = null; // Free the connection
    "#});

    assert_eq!(
        indoc! {r#"
            string(6) "value1"
        "#},
        output
    );
}

#[test]
fn test_rollback_transaction() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_transaction";

        // Test rollback
        $transaction = new RocksDBTransaction($dbPath);
        $transaction->put("key2", "value2");
        $transaction->rollback();
        $transaction = null; // Free the connection

        $db = new RocksDBTransaction($dbPath);
        $value = $db->get("key2");
        var_dump($value); // Expecting null
        $db = null; // Free the connection
    "#});

    assert_eq!(
        indoc! {r#"
            NULL
        "#},
        output
    );
}

#[test]
fn test_savepoint_transaction() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_transaction";

        // Test savepoint
        $transaction = new RocksDBTransaction($dbPath);
        $transaction->put("key3", "value3");
        $transaction->setSavepoint();
        $transaction->put("key4", "value4");
        $transaction->rollbackToSavepoint();
        $transaction->commit();
        $transaction = null; // Free the connection

        $db = new RocksDBTransaction($dbPath);
        $value3 = $db->get("key3");
        $value4 = $db->get("key4");
        var_dump($value3); // Expecting value3
        var_dump($value4); // Expecting null
        $db = null; // Free the connection
    "#});

    assert_eq!(
        indoc! {r#"
            string(6) "value3"
            NULL
        "#},
        output
    );
}