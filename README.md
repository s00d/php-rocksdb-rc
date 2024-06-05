# php-rocksdb-rc

A PHP extension for interacting with RocksDB, built with Rust. This extension provides a simple interface for RocksDB operations such as putting, getting, deleting, and managing column families.

## Overview

`php-rocksdb-rc` is a PHP extension that allows you to use RocksDB, a high-performance embedded database for key-value data, directly from your PHP applications. This extension is written in Rust using the `ext-php-rs` and `rust-rocksdb` crates to provide a seamless integration between PHP and RocksDB.

## Features

- Basic CRUD operations on RocksDB
- Support for column families
- TTL support for key-value pairs
- Advanced options like flushing and repairing the database
- Backup and restore functionality
- Write batch operations
- Snapshot support
- Transaction support

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

#### Install Rust and Cargo

1. **On Linux and macOS**:

   The recommended way to install Rust is via `rustup`, a toolchain installer for Rust.

    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

   Follow the on-screen instructions to complete the installation. After installation, you may need to restart your terminal or source your profile:

    ```sh
    source $HOME/.cargo/env
    ```

2. **On Windows**:

   Download and run the `rustup-init.exe` installer from the [official Rust website](https://www.rust-lang.org/tools/install). Follow the on-screen instructions to complete the installation.

   After installation, open a new command prompt or PowerShell window and ensure that `cargo` and `rustc` are in your PATH by running:

    ```sh
    rustc --version
    cargo --version
    ```

#### Clone the Repository and Build the Extension

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

   - **On Linux**:

       ```sh
       cp target/release/libphp_rocksdb_rc.so /usr/lib/php/extensions/
       ```

   - **On macOS**:

       ```sh
       cp target/release/libphp_rocksdb_rc.dylib /usr/local/lib/php/extensions/
       ```

   - **On Windows**:

       ```sh
       copy target\release\php_rocksdb_rc.dll C:\path\to\php\ext\
       ```

4. Add the following line to your `php.ini` file:

   - **On Linux and macOS**:

       ```ini
       extension=libphp_rocksdb_rc.so
       ```

   - **On Windows**:

       ```ini
       extension=php_rocksdb_rc.dll
       ```

5. Restart your web server or PHP-FPM to load the extension.

### Additional Notes

- Ensure that the path to the PHP extensions directory is correct. You can find the extension directory by running `php -i | grep extension_dir`.
- If you encounter any issues during the installation of Rust, refer to the [official Rust installation guide](https://www.rust-lang.org/tools/install) for troubleshooting tips.
- Make sure your PHP installation is compatible with the extension. You can check your PHP version by running `php -v`.

By following these steps, you should be able to build and install the PHP extension from source successfully.


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
$db->createColumnFamily("new_cf");

// Put a value in a column family
$db->put("key2", "value2", "new_cf");

// Get a value from a column family
$value = $db->get("key2", "new_cf");
echo $value; // Outputs: value2

// List column families
$column_families = RocksDB::listColumnFamilies("/path/to/db");
print_r($column_families);

// Drop a column family
$db->dropColumnFamily("new_cf");

// Flush the database
$db->flush();

// Repair the database
RocksDB::repair("/path/to/db");

// Close the database
$db->close();
?>
```

This example demonstrates basic put and get operations in RocksDB.

```php
<?php
$dbPath = __DIR__ . "/temp/testdb";
$db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
$db->put("key1", "value1");
$db = null; // Free the connection

$db = new RocksDB($dbPath, 3600);
$value = $db->get("key1");
var_dump($value); // Outputs: string(6) "value1"
$db = null; // Free the connection
?>
```

### Example: Delete Operation

This example demonstrates how to delete a key-value pair from RocksDB.

```php
<?php
$dbPath = __DIR__ . "/temp/testdb";
$db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
$db->put("key1", "value1");
$db->delete("key1");
$db = null; // Free the connection

$db = new RocksDB($dbPath, 3600);
$value = $db->get("key1");
var_dump($value); // Outputs: NULL
$db = null; // Free the connection
?>
```

### Example: Iterator Usage

This example demonstrates how to use an iterator to traverse key-value pairs in RocksDB.

```php
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
var_dump($result); // Outputs an array with all key-value pairs
$db = null; // Free the connection
?>
```

### Example: Backup and Restore

This example demonstrates how to create a backup of the RocksDB database and retrieve information about the backups.

```php
<?php
$dbPath = __DIR__ . "/temp/testdb_backup";
$backupPath = __DIR__ . "/temp/testdb_backup_files";

// Create and use RocksDB instance
$db = new RocksDB($dbPath, 3600); // 3600 seconds TTL
$db->put("key1", "value1");

// Initialize backup engine and create a backup
$backup = new \RocksDBBackup($dbPath, 3600); // 3600 seconds TTL
$backup->init($backupPath);
$backup->create();

// Get backup info
$info = $backup->info();
print_r($info); // Outputs information about the backups

// Restore from backup
$restorePath = __DIR__ . "/temp/testdb_restore";
$backup->restore(1, $restorePath); // Restore the first backup

// Verify the restored data
$restoredDb = new RocksDB($restorePath, 3600);
$value = $restoredDb->get("key1");
var_dump($value); // Outputs: string(6) "value1"

// Cleanup
$db = null;
$restoredDb = null;
?>
```

### Example: Write Batch Operations

This example demonstrates how to use write batch operations to perform multiple writes in a single batch.

```php
<?php
$dbPath = __DIR__ . "/temp/testdb_write_batch";
$db = new RocksDB($dbPath, 3600); // 3600 seconds TTL

// Initialize write batch
$writeBatch = new \RocksDBWriteBatch($dbPath, 3600); // 3600 seconds TTL
$writeBatch->start();
$writeBatch->put("key1", "value1");
$writeBatch->put("key2", "value2");
$writeBatch->delete("key1");
$writeBatch->write(); // Write the batch to the database

// Verify the data
$value1 = $db->get("key1");
$value2 = $db->get("key2");
var_dump($value1); // Outputs: NULL (since key1 was deleted)
var_dump($value2); // Outputs: string(6) "value2"

// Cleanup
$db = null;
?>
```

### Example: Transactions

This example demonstrates how to use transactions to ensure atomicity of multiple operations.

```php
<?php
$dbPath = __DIR__ . "/temp/testdb_transaction";
$db = new RocksDB($dbPath, 3600); // 3600 seconds TTL

// Initialize transaction
$transaction = new \RocksDBTransaction($dbPath, 3600); // 3600 seconds TTL
$transaction->start();
$transaction->put("key1", "value1");
$transaction->put("key2", "value2");
$transaction->delete("key1");
$transaction->commit(); // Commit the transaction

// Verify the data
$value1 = $db->get("key1");
$value2 = $db->get("key2");
var_dump($value1); // Outputs: NULL (since key1 was deleted)
var_dump($value2); // Outputs: string(6) "value2"

// Cleanup
$db = null;
?>
```

### Example: Snapshots

This example demonstrates how to use snapshots to capture the state of the database at a specific point in time.

```php
<?php
$dbPath = __DIR__ . "/temp/testdb_snapshot";
$db = new RocksDB($dbPath, 3600); // 3600 seconds TTL

// Put some initial data
$db->put("key1", "value1");
$db->put("key2", "value2");

// Create a snapshot
$snapshot = new \RocksDBSnapshot($dbPath, 3600); // 3600 seconds TTL
$snapshot->create();

// Modify the database after taking the snapshot
$db->put("key1", "new_value1");
$db->delete("key2");

// Verify the data in the snapshot
$snapshotValue1 = $snapshot->get("key1");
$snapshotValue2 = $snapshot->get("key2");
var_dump($snapshotValue1); // Outputs: string(6) "value1" (original value)
var_dump($snapshotValue2); // Outputs: string(6) "value2" (original value)

// Verify the current data in the database
$currentValue1 = $db->get("key1");
$currentValue2 = $db->get("key2");
var_dump($currentValue1); // Outputs: string(10) "new_value1"
var_dump($currentValue2); // Outputs: NULL (since key2 was deleted)

// Cleanup
$db = null;
$snapshot = null;
?>
```


### Detailed API

#### `__construct(path: String, ttl_secs: Option<u64>)`
Creates a new RocksDB instance with the specified path and TTL.

```php
<?php
$db = new RocksDB("/path/to/db", 3600); // 3600 seconds TTL
?>
```

#### `put(key: String, value: String, cf_name: Option<String>)`
Inserts a key-value pair into the database.

```php
<?php
$db->put("key1", "value1");
$db->put("key2", "value2", "new_cf"); // Using column family
?>
```

#### `get(key: String, cf_name: Option<String>)`
Retrieves the value associated with the given key.

```php
<?php
$value = $db->get("key1");
echo $value; // Outputs: value1

$value = $db->get("key2", "new_cf"); // From column family
echo $value; // Outputs: value2
?>
```

#### `merge(key: String, value: String, cf_name: Option<String>)`
Merges a value into the database using JSON Patch.

```php
<?php
$db->merge("json_obj_key", '[ { "op": "replace", "path": "/employees/1/first_name", "value": "lucy" } ]');
$db->merge("json_obj_key", '[ { "op": "replace", "path": "/employees/0/last_name", "value": "dow" } ]');
?>
```


This method uses JSON Patch to update the JSON object in the database. For more details on JSON Patch, refer to  [RFC 6902](https://datatracker.ietf.org/doc/html/rfc6902).

#### `delete(key: String, cf_name: Option<String>)`
Deletes the key-value pair associated with the given key.

```php
<?php
$db->delete("key1");
$db->delete("key2", "new_cf"); // From column family
?>
```

#### `listColumnFamilies(path: String)`
Lists all column families in the database.

```php
<?php
$column_families = RocksDB::listColumnFamilies("/path/to/db");
print_r($column_families);
?>
```

#### `createColumnFamily(cf_name: String)`
Creates a new column family with the specified name.

```php
<?php
$db->createColumnFamily("new_cf");
?>
```

#### `dropColumnFamily(cf_name: String)`
Drops the column family with the specified name.

```php
<?php
$db->dropColumnFamily("new_cf");
?>
```

#### `getProperty(property: String, cf_name: Option<String>)`
Retrieves a database property.

```php
<?php
$property = $db->getProperty("rocksdb.stats");
echo $property;

$property = $db->getProperty("rocksdb.stats", "new_cf"); // From column family
echo $property;
?>
```

#### `flush(cf_name: Option<String>)`
Flushes all memtable data to SST files.

```php
<?php
$db->flush();
$db->flush("new_cf"); // Flush column family
?>
```

#### `repair(path: String)`
Repairs a RocksDB database at the specified path.

```php
<?php
RocksDB::repair("/path/to/db");
?>
```

#### `close()`
Closes the RocksDB instance.

```php
<?php
$db->close();
?>
```

#### `all(cf_name: Option<String>)`
Returns all key-value pairs in the database or column family.

```php
<?php
$data = $db->all();
print_r($data);

$data = $db->all("new_cf"); // From column family
print_r($data);
?>
```

#### `keys(cf_name: Option<String>)`
Returns all keys in the database or column family.

```php
<?php
$keys = $db->keys();
print_r($keys);

$keys = $db->keys("new_cf"); // From column family
print_r($keys);
?>
```

### Iterator Methods

#### `seekToFirst()`
Seeks to the first key in the database or column family.

```php
<?php
$db->seekToFirst();
?>
```

#### `seekToLast()`
Seeks to the last key in the database or column family.

```php
<?php
$db->seekToLast();
?>
```

#### `seek(key: String)`
Seeks to the specified key in the database or column family.

```php
<?php
$db->seek("key1");
?>
```

#### `seekForPrev(key: String)`
Seeks to the specified key or previous key in the database or column family.

```php
<?php
$db->seekForPrev("key1");
?>
```

#### `valid()`
Checks if the current iterator position is valid.

```php
<?php
$isValid = $db->valid();
echo $isValid ? 'true' : 'false';
?>
```

#### `next()`
Moves to the next key-value pair in the database or column family.

```php
<?php
$kv = $db->next();
print_r($kv);
?>
```

#### `prev()`
Moves to the previous key-value pair in the database or column family.

```php
<?php
$kv = $db->prev();
print_r($kv);
?>
```

#### `compact_range(start: Option<String>, end: Option<String>, cf_name: Option<String>)`
Compacts the key-value pairs in the specified range within the database or column family.

```php
<?php
$db->compact_range("key_start", "key_end");
$db->compact_range("key_start", "key_end", "new_cf"); // In column family
?>
```

#### `get_live_files()`
Returns the names of the live SST files in the database.

```php
<?php
$live_files = $db->get_live_files();
print_r($live_files);
?>
```

#### `set_options(options: HashMap<String, String>, cf_name: Option<String>)`
Sets the database options.

```php
<?php
$options = [
    "write_buffer_size" => "4194304",
    "max_write_buffer_number" => "3",
];
$db->set_options($options);
$db->set_options($options, "new_cf"); // For column family
?>
```

#### `set_compression(compression_type: String, cf_name: Option<String>)`
Sets the compression type for the database or column family.

```php
<?php
$db->set_compression("snappy");
$db->set_compression("zlib", "new_cf"); // For column family
?>
```

This method supports the following compression types: "none", "snappy", "zlib", "bzip2", "lz4", "lz4hc", "zstd".

#### `set_write_buffer_size(size: usize, cf_name: Option<String>)`
Sets the size of the write buffer for the database or column family.

```php
<?php
$db->set_write_buffer_size(4194304);
$db->set_write_buffer_size(4194304, "new_cf"); // For column family
?>
```

This method sets the amount of data to build up in memory (backed by an unsorted log on disk) before converting to a sorted on-disk file.

#### `set_cache_size(size: usize, cf_name: Option<String>)`
Sets the size of the block cache for the database or column family.

```php
<?php
$db->set_cache_size(8388608);
$db->set_cache_size(8388608, "new_cf"); // For column family
?>
```

This method sets the amount of memory to use for the block cache, which is used to accelerate the read operations by caching the data blocks in memory.

### Backup Methods

#### `__construct(path: String, ttl_secs: Option<u64>)`
Creates a new RocksDBBackup instance.

```php
<?php
$backup = new \RocksDBBackup("/path/to/db", 3600); // 3600 seconds TTL
?>
```

#### `init(backup_path: String)`
Initializes the backup engine with the specified path.

```php
<?php
$backup->init("/path/to/backup");
?>
```

#### `create()`
Creates a backup of the database.

```php
<?php
$backup->init("/path/to/backup");
$backup->create();
?>
```

#### `info()`
Returns information about the backups.

```php
<?php
$backup->init("/path/to/backup");
$info = $backup->info();
print_r($info);
?>
```

#### `purgeOld(num_backups_to_keep: usize)`
Purges old backups, keeping the specified number of backups.

```php
<?php
$backup->init("/path/to/backup");
$backup->purgeOld(2);
?>
```

#### `restore(backup_id: u32, restore_path: String)`
Restores the database from a backup.

```php
<?php
$backup->init("/path/to/backup");
$backup->restore(1, "/path/to/restore");
?>
```

### Write Batch Methods

#### `__construct(path: String, ttl_secs: Option<u64>)`
Creates a new RocksDBWriteBatch instance.

```php
<?php
$write_batch = new \RocksDBWriteBatch("/path/to/db", 3600); // 3600 seconds TTL
?>
```

#### `start()`
Starts a new write batch.

```php
<?php
$write_batch->start();
?>
```

#### `put(key: String, value: String, cf_name: Option<String>)`
Puts a key-value pair into the current write batch.

```php
<?php
$write_batch->start();
$write_batch->put("key1", "value1");
$write_batch->put("key2", "value2", "new_cf"); // Using column family
?>
```

#### `merge(key: String, value: String, cf_name: Option<String>)`
Merges a value into the current write batch.

```php
<?php
$write_batch->start();
$write_batch->merge("json_obj_key", "employees[1].first_name = lucy");
$write_batch->merge("json_obj_key", "employees[0].last_name = dow", "new_cf"); // Using column family
?>
```

#### `delete(key: String, cf_name: Option<String>)`
Deletes a key-value pair from the current write batch.

```php
<?php
$write_batch->start();
$write_batch->delete("key1");
$write_batch->delete("key2", "new_cf"); // From column family
?>
```

#### `write()`
Writes the current write batch to the database.

```php
<?php
$write_batch->start();
$write_batch->write();
?>
```

#### `clear()`
Clears the current write batch.

```php
<?php
$write_batch->start();
$write_batch->clear();
?>
```

#### `destroy()`
Destroys the current write batch.

```php
<?php
$write_batch->start();
$write_batch->destroy();
?>
```

### Transaction Methods

#### `__construct(path: String, ttl_secs: Option<u64>)`
Creates a new RocksDBTransaction instance.

```php
<?php
$transaction = new \RocksDBTransaction("/path/to/db", 3600); // 3600 seconds TTL
?>
```

#### `start()`
Starts a new transaction.

```php
<?php
$transaction->start();
?>
```

#### `commit()`
Commits the current transaction.

```php
<?php
$transaction->commit();
?>
```

#### `rollback()`
Rolls back the current transaction.

```php
<?php
$transaction->rollback();
?>
```

#### `setSavepoint()`
Sets a savepoint within the current transaction.

```php
<?php
$transaction->setSavepoint();
?>
```

#### `rollbackToSavepoint()`
Rolls back the transaction to the last savepoint.

```php
<?php
$transaction->rollbackToSavepoint();
?>
```

#### `put(key: String, value: String, cf_name: Option<String>)`
Puts a key-value pair into the current transaction.

```php
<?php
$transaction->put("key1", "value1");
$transaction->put("key2", "value2", "new_cf"); // Using column family
?>
```

#### `get(key: String, cf_name: Option<String>)`
Gets the value associated with the given key within the current transaction.

```php
<?php
$value = $transaction->get("key1");
echo $value; // Outputs: value1

$value = $transaction->get("key2", "new_cf"); // From column family
echo $value; // Outputs: value2
?>
```

#### `delete(key: String, cf_name: Option<String>)`
Deletes a key-value pair within the current transaction.

```php
<?php
$transaction->delete("key1");
$transaction->delete("key2", "new_cf"); // From column family
?>
```

#### `merge(key: String, value: String, cf_name: Option<String>)`
Merges a value within the current transaction.

```php
<?php
$transaction->merge("json_obj_key", "employees[1].first_name = lucy");
$transaction->merge("json_obj_key", "employees[0].last_name = dow", "new_cf"); // Using column family
?>
```

## Important Note

Before creating a new instance of any class (e.g., `RocksDB`, `RocksDBBackup`, `RocksDBWriteBatch`, `RocksDBTransaction`), ensure to destroy the previous instance to free up the database connection.

```php
<?php
$db = new \RocksDB($dbPath);
$db = null; // Free the connection

$iterator = new \RocksDBIterator($dbPath); // Now you can create a new instance
?>
```

## Enabling Autocompletion in PhpStorm

To enable autocompletion for the `php_rocksdb_rc` extension in PhpStorm, follow these steps:

1. **Download the Stub File**

   Download the `.php_rocksdb_rc.php` file from [here](.php_rocksdb_rc.php).

2. **Place the Stub File in Your Project**

   Save the `.php_rocksdb_rc.php` file in your project directory.

After completing these steps, you should have autocompletion support for the `php_rocksdb_rc` extension in PhpStorm.

## Contributing

Contributions are welcome! Please submit pull requests or issues on the [GitHub repository](https://github.com/yourusername/php-rocksdb-rc).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

This project uses the following open-source libraries:
- [ext-php-rs](https://github.com/davidcole1340/ext-php-rs)
- [rust-rocksdb](https://github.com/rust-rocksdb/rust-rocksdb)

## Conclusion

The `php-rocksdb-rc` extension provides a powerful and efficient way to interact with RocksDB from PHP. With support for basic CRUD operations, column families, TTL, backups, write batches, transactions, and snapshots, it offers a comprehensive set of features for managing key-value data in PHP applications. The provided examples demonstrate how to use these features effectively. Happy coding!