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
        $dbPath = __DIR__ . "/temp/testdb_put";
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