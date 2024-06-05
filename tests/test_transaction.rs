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
