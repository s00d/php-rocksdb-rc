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
fn test_seek_to_last() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_seek_last";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->put("key2", "value2");
        $db->put("key3", "value3");

        $db->seekToLast();
        $res = $db->prev();
        var_dump($res);

        $db = null; // Free the connection
    "#});
    assert_eq!(
        indoc! {r#"
            array(2) {
              ["key"]=>
              string(4) "key3"
              ["value"]=>
              string(6) "value3"
            }
        "#},
        output
    );
}

#[test]
fn test_seek() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_seek";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->put("key2", "value2");
        $db->put("key3", "value3");

        $db->seek("key2");
        $res = $db->next();
        var_dump($res);

        $db = null; // Free the connection
    "#});
    assert_eq!(
        indoc! {r#"
            array(2) {
              ["key"]=>
              string(4) "key2"
              ["value"]=>
              string(6) "value2"
            }
        "#},
        output
    );
}

#[test]
fn test_valid() {
    setup();
    let output = php_request(indoc! { r#"
        <?php
        $dbPath = __DIR__ . "/temp/testdb_valid";
        $db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
        $db->put("key1", "value1");
        $db->put("key2", "value2");
        $db->put("key3", "value3");

        $db->seekToFirst();
        $isValid = $db->valid();
        var_dump($isValid);

        $db = null; // Free the connection
    "#});
    assert_eq!(
        indoc! {r#"
            bool(true)
        "#},
        output
    );
}
