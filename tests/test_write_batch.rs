use std::thread::sleep;
use std::time;

mod common;
use common::php_request;

fn setup() {
    common::setup();
    sleep(time::Duration::from_secs(1));
}

#[test]
fn test_write_batch() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_write_batch";
        $write_batch = new RocksDBWriteBatch($dbPath, 3600); // 3600 seconds TTL
        $write_batch->start();
        $write_batch->put("key1", "value1");
        $write_batch->put("key2", "value2");
        $write_batch->write();
        $write_batch = null; // Free the connection

        $db = new RocksDB($dbPath, 3600);
        $value1 = $db->get("key1");
        $value2 = $db->get("key2");
        echo $value1 . "\n" . $value2;
        $db = null; // Free the connection
    "#,
    );
    let expected_output = "value1\nvalue2";
    assert_eq!(output.trim(), expected_output);
}
