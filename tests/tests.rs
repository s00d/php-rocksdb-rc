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
        $dbPath = __DIR__ . "/temp/testdb_all";
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
        $dbPath = __DIR__ . "/temp/testdb_get";
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
        $dbPath = __DIR__ . "/temp/testdb_delete";
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
fn test_merge() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
            $dbPath = __DIR__ . "/temp/testdb_patch";
            $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL

            // Add initial JSON object
            $initial_json = json_encode([
                "employees" => [
                    ["first_name" => "john", "last_name" => "doe"],
                    ["first_name" => "adam", "last_name" => "smith"]
                ]
            ]);
            $db->put("json_obj_key", $initial_json);

            // Perform merge to update JSON object using JSON Patch
            $patch1 = json_encode([
                ["op" => "replace", "path" => "/employees/1/first_name", "value" => "lucy"]
            ]);
            $db->merge("json_obj_key", $patch1);

            $patch2 = json_encode([
                ["op" => "replace", "path" => "/employees/0/last_name", "value" => "dow"]
            ]);
            $db->merge("json_obj_key", $patch2);

            // Release the connection
            $db = null;

            // Reopen the database
            $db = new RocksDB($dbPath, 3600);

            // Get the value after merge
            $value = $db->get("json_obj_key");
            var_dump($value);

            // Release the connection
            $db = null;
            ?>

    "#});
    assert_eq!(
        indoc! {r#"
            string(97) "{"employees":[{"first_name":"john","last_name":"dow"},{"first_name":"lucy","last_name":"smith"}]}"

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
        $dbPath = __DIR__ . "/temp/testdb_backup";
        $backupPath = __DIR__ . "/temp/backup1";
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
fn test_restore_backup() {
    setup();
    let output = php_request(indoc! { r#"
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
        var_dump($value);
        $backup = null; // Free the connection
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
fn test_write_batch() {
    setup();
    let output = php_request(indoc! { r#"
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

#[test]
fn test_create_column_family() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_cf";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->createColumnFamily("new_cf");
        $cfs = $db->listColumnFamilies($dbPath);
         $db->dropColumnFamily("new_cf");
        var_dump($cfs);
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
    assert!(!output.contains("new_cf"));
}

#[test]
fn test_get_property() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_get_property";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $property = $db->getProperty("rocksdb.stats");
        var_dump($property);
        $db = null; // Free the connection
    "#});
    assert!(output.contains("** DB Stats **"));
}

#[test]
fn test_flush() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_flush";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->flush();
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
fn test_repair() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_repair";
        RocksDB::repair($dbPath);
        var_dump(true);
    "#});
    assert_eq!(
        indoc! {r#"
            bool(true)
        "#},
        output
    );
}

#[test]
fn test_keys() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_keys";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->put("key2", "value2");
        $keys = $db->keys();
        sort($keys);
        var_dump($keys);
        $db = null; // Free the connection
    "#});
    assert_eq!(
        indoc! {r#"
            array(2) {
              [0]=>
              string(4) "key1"
              [1]=>
              string(4) "key2"
            }
        "#},
        output
    );
}

#[test]
fn test_all() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_all";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->put("key2", "value2");
        $all = $db->all();
        ksort($all);
        var_dump($all);
        $db = null; // Free the connection
    "#});
    assert_eq!(
        indoc! {r#"
            array(2) {
              ["key1"]=>
              string(6) "value1"
              ["key2"]=>
              string(6) "value2"
            }
        "#},
        output
    );
}
