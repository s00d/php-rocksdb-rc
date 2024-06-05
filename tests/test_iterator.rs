use std::thread::sleep;
use std::time;

mod common;
use common::php_request;

fn setup() {
    common::setup();
    sleep(time::Duration::from_secs(1));
}

#[test]
fn test_iterator() {
    setup();
    let output = php_request(
        r#"
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
        echo json_encode($result);
        $db = null; // Free the connection
    "#,
    );

    assert_eq!(
        output.trim(),
        r#"{"key_ggg":"value_b","key_hhh":"value_c","key_vvv":"value_a"}"#
    );
}

#[test]
fn test_seek_to_last() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_seek_last";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->put("key2", "value2");
        $db->put("key3", "value3");

        $db->seekToLast();
        $res = $db->prev();
        echo json_encode($res);
        $db = null; // Free the connection
    "#,
    );
    assert_eq!(output.trim(), r#"{"key":"key3","value":"value3"}"#);
}

#[test]
fn test_seek() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_seek";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->put("key2", "value2");
        $db->put("key3", "value3");

        $db->seek("key2");
        $res = $db->next();
        echo json_encode($res);
        $db = null; // Free the connection
    "#,
    );
    assert_eq!(output.trim(), r#"{"key":"key2","value":"value2"}"#);
}

#[test]
fn test_valid() {
    setup();
    let output = php_request(
        r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_valid";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->put("key2", "value2");
        $db->put("key3", "value3");

        $db->seekToFirst();
        $isValid = $db->valid();
        echo $isValid ? 'true' : 'false';
        $db = null; // Free the connection
    "#,
    );
    assert_eq!(output.trim(), "true");
}
