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
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_put";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db = null; // Free the connection

        $db = new RocksDB($dbPath, 3600);
        $value = $db->get("key1");
        echo $value;
        $db = null; // Free the connection
    "#,
    );
    assert_eq!(output.trim(), "value1");
}

#[test]
fn test_get() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_get";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db = null; // Free the connection

        $db = new RocksDB($dbPath, 3600);
        $value = $db->get("key1");
        echo $value;
        $db = null; // Free the connection
    "#,
    );
    assert_eq!(output.trim(), "value1");
}

#[test]
fn test_delete() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_delete";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->delete("key1");
        $db = null; // Free the connection

        $db = new RocksDB($dbPath, 3600);
        $value = $db->get("key1");
        echo $value ? $value : 'NULL';
        $db = null; // Free the connection
    "#,
    );
    assert_eq!(output.trim(), "NULL");
}

#[test]
fn test_merge() {
    setup();
    let output = php_request(
        r#"
        <?php
            $dbPath = __DIR__ . "/temp/testdb_merge";
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
            echo $value;

            // Release the connection
            $db = null;
            ?>

    "#,
    );
    assert_eq!(
        r#"{"employees":[{"first_name":"john","last_name":"dow"},{"first_name":"lucy","last_name":"smith"}]}"#,
        output.trim()
    );
}

#[test]
fn test_get_property() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_get_property";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $property = $db->getProperty("rocksdb.stats");
        echo $property;
        $db = null; // Free the connection
    "#,
    );
    assert!(output.contains("** DB Stats **"));
}

#[test]
fn test_flush() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_flush";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->flush();
        $db = null; // Free the connection

        $db = new RocksDB($dbPath, 3600);
        $value = $db->get("key1");
        echo $value;
        $db = null; // Free the connection
    "#,
    );
    assert_eq!(output.trim(), "value1");
}

#[test]
fn test_repair() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_repair";
        RocksDB::repair($dbPath);
        echo true;
    "#,
    );
    assert_eq!(output.trim(), "1");
}

#[test]
fn test_keys() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_keys";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->put("key2", "value2");
        $keys = $db->keys();
        sort($keys);
        echo json_encode($keys);
        $db = null; // Free the connection
    "#,
    );
    assert_eq!(output.trim(), r#"["key1","key2"]"#);
}

#[test]
fn test_all() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_all";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->put("key2", "value2");
        $all = $db->all();
        ksort($all);
        echo json_encode($all);
        $db = null; // Free the connection
    "#,
    );
    assert_eq!(output.trim(), r#"{"key1":"value1","key2":"value2"}"#);
}
