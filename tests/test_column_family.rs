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
fn test_create_column_family() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_cf";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->createColumnFamily("new_cf");
        $cfs = $db->listColumnFamilies($dbPath);
        var_dump($cfs);
        $db->dropColumnFamily("new_cf");
        $db = null; // Free the connection
    "#});
    assert!(output.contains("new_cf"));
}

#[test]
fn test_drop_column_family() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_drop_cf";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->createColumnFamily("new_cf_drop");
        $db->dropColumnFamily("new_cf_drop");
        $cfs = $db->listColumnFamilies($dbPath);
        var_dump($cfs);
        $db = null; // Free the connection
    "#});
    assert!(!output.contains("new_cf_drop"));
}

#[test]
fn test_list_column_families() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_list_cf";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->createColumnFamily("cf1");
        $db->createColumnFamily("cf2");
        $cfs = $db->listColumnFamilies($dbPath);
        var_dump($cfs);
        $db = null; // Free the connection
    "#});
    assert!(output.contains("cf1"));
    assert!(output.contains("cf2"));
}
