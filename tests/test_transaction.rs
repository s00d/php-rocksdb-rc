use std::thread::sleep;
use std::time;

mod common;
use common::php_request;

fn setup() {
    common::setup();
    sleep(time::Duration::from_secs(1));
}

#[test]
fn test_commit_transaction() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_transaction";

        // Test commit
        $transaction = new RocksDBTransaction($dbPath);
        $transaction->put("key1", "value1");
        $transaction->commit();
        $transaction = null; // Free the connection

        $db = new RocksDBTransaction($dbPath);
        $value = $db->get("key1");
        echo $value; // Expecting value1
        $db = null; // Free the connection
    "#,
    );

    assert_eq!(output.trim(), "value1");
}

#[test]
fn test_rollback_transaction() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_transaction";

        // Test rollback
        $transaction = new RocksDBTransaction($dbPath);
        $transaction->put("key2", "value2");
        $transaction->rollback();
        $transaction = null; // Free the connection

        $db = new RocksDBTransaction($dbPath);
        $value = $db->get("key2");
        echo $value ? $value : 'NULL'; // Expecting null
        $db = null; // Free the connection
    "#,
    );

    assert_eq!(output.trim(), "NULL");
}

#[test]
fn test_savepoint_transaction() {
    setup();
    let output = php_request(
        r#"
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
        echo $value3 . "\n" . ($value4 ? $value4 : 'NULL'); // Expecting value3 and null
        $db = null; // Free the connection
    "#,
    );

    let expected_output = "value3\nNULL";
    assert_eq!(output.trim(), expected_output);
}
