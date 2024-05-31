# php-rocksdb-rc

A PHP extension for interacting with RocksDB, built with Rust. This extension provides a simple interface for RocksDB operations such as putting, getting, deleting, and managing column families.

## Overview

`php-rocksdb-rc` is a PHP extension that allows you to use RocksDB, a high-performance embedded database for key-value data, directly from your PHP applications. This extension is written in Rust using the `ext-php-rs` and `rust-rocksdb` crates to provide a seamless integration between PHP and RocksDB.

## Features

- Basic CRUD operations on RocksDB
- Support for column families
- TTL support for key-value pairs
- Advanced options like flushing and repairing the database

## Installation

### Pre-built Binaries

You can download pre-built binaries from the [releases page](https://github.com/yourusername/php-rocksdb-rc/releases). Download the appropriate binary for your system and PHP version.

#### For Linux:

1. Download the `.so` file for your PHP version and architecture.
2. Place the file in your PHP extensions directory (usually `/usr/lib/php/extensions`).
3. Add the following line to your `php.ini` file:

    ```ini
    extension=php_rocksdb_rc.so
    ```

4. Restart your web server or PHP-FPM to load the extension.

#### For macOS:

1. Download the `.dylib` file for your PHP version and architecture.
2. Place the file in your PHP extensions directory (usually `/usr/local/lib/php/extensions`).
3. Add the following line to your `php.ini` file:

    ```ini
    extension=php_rocksdb_rc.dylib
    ```

4. Restart your web server or PHP-FPM to load the extension.

### Building from Source

To build the extension from source, you will need Rust and Cargo installed. Follow the steps below:

1. Clone the repository:

    ```sh
    git clone https://github.com/yourusername/php-rocksdb-rc.git
    cd php-rocksdb-rc
    ```

2. Build the extension:

    ```sh
    cargo build --release
    ```

3. Copy the built library to your PHP extensions directory:

    - On Linux:

        ```sh
        cp target/release/libphp_rocksdb_rc.so /usr/lib/php/extensions/
        ```

    - On macOS:

        ```sh
        cp target/release/libphp_rocksdb_rc.dylib /usr/local/lib/php/extensions/
        ```

4. Add the following line to your `php.ini` file:

    ```ini
    extension=php_rocksdb_rc.so
    ```

   or on macOS:

    ```ini
    extension=php_rocksdb_rc.dylib
    ```

5. Restart your web server or PHP-FPM to load the extension.

## Usage

Here are some examples of how to use the `php-rocksdb-rc` extension:

### Creating and Using a RocksDB Instance

```php
<?php

$db = new RocksDB("/path/to/db", 3600); // 3600 seconds TTL

// Put a value
$db->put("key1", "value1");

// Get a value
$value = $db->get("key1");
echo $value; // Outputs: value1

// Delete a value
$db->delete("key1");

// Create a column family
$db->create_column_family("new_cf");

// Put a value in a column family
$db->put_cf("key2", "value2", "new_cf");

// Get a value from a column family
$value = $db->get_cf("key2", "new_cf");
echo $value; // Outputs: value2

// List column families
$column_families = RocksDB::list_column_families("/path/to/db");
print_r($column_families);

// Drop a column family
$db->drop_column_family("new_cf");

// Flush the database
$db->flush();

// Repair the database
RocksDB::repair("/path/to/db");

// Close the database
$db->close();
?>
```

### Detailed API

- `__construct(path: String, ttl_secs: u64)`: Creates a new RocksDB instance with the specified path and TTL.
- `put(key: String, value: String)`: Inserts a key-value pair into the database.
- `put_cf(key: String, value: String, cf_name: String)`: Inserts a key-value pair into the specified column family.
- `get(key: String)`: Retrieves the value associated with the given key.
- `get_cf(key: String, cf_name: String)`: Retrieves the value associated with the given key from the specified column family.
- `delete(key: String)`: Deletes the key-value pair associated with the given key.
- `delete_cf(key: String, cf_name: String)`: Deletes the key-value pair associated with the given key from the specified column family.
- `list_column_families(path: String)`: Lists all column families in the database.
- `create_column_family(cf_name: String)`: Creates a new column family with the specified name.
- `drop_column_family(cf_name: String)`: Drops the column family with the specified name.
- `get_property(property: String)`: Retrieves a database property.
- `get_property_cf(property: String, cf_name: String)`: Retrieves a database property from the specified column family.
- `flush()`: Flushes all memtable data to SST files.
- `repair(path: String)`: Repairs a RocksDB database at the specified path.
- `close()`: Closes the RocksDB instance.

## Contributing

Contributions are welcome! Please submit pull requests or issues on the [GitHub repository](https://github.com/yourusername/php-rocksdb-rc).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

This project uses the following open-source libraries:
- [ext-php-rs](https://github.com/davidcole1340/ext-php-rs)
- [rust-rocksdb](https://github.com/rust-rocksdb/rust-rocksdb)
