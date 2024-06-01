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
$db->createColumnFamily("new_cf");

// Put a value in a column family
$db->put("key2", "value2", "new_cf");

// Get a value from a column family
$value = $db->get("key2", "new_cf");
echo $value; // Outputs: value2

// List column families
$column_families = $db->listColumnFamilies("/path/to/db");
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
Merges a value into the database.

```php
<?php
$db->merge("json_obj_key", "{ employees: [ {first_name: john, last_name: doe}, {first_name: adam, last_name: smith}] }");
$db->merge("json_obj_key", "employees[1].first_name = lucy");
$db->merge("json_obj_key", "employees[0].last_name = dow");
?>
```

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

#### `__construct(path: String, ttl_secs: Option<u64>)`
Creates a new RocksDBIterator instance.

```php
<?php
$iterator = new \RocksDBIterator("/path/to/db", 3600); // 3600 seconds TTL
?>
```

#### `iterator(cf_name: Option<String>)`
Initializes an iterator for the database or column family.

```php
<?php
$iterator = new \RocksDBIterator($dbPath);
$iterator->iterator();
while (true) {
    $batch = $iterator->next(2);
    if (empty($batch)) {
        break;
    }
    print_r($batch);
}
?>
```

#### `next(batch_size: usize)`
Gets the next batch of key-value pairs from the iterator.

```php
<?php
$iterator->iterator();
while (true) {
    $batch = $iterator->next(2);
    if (empty($batch)) {
        break;
    }
    print_r($batch);
}
?>
```

#### `reset()`
Resets the iterator.

```php
<?php
$iterator->reset();
?>
```

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

#### `purge_old(num_backups_to_keep: usize)`
Purges old backups, keeping the specified number of backups.

```php
<?php
$backup->init("/path/to/backup");
$backup->purge_old(2);
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

### Snapshot Methods

#### `__construct(path: String, ttl_secs: Option<u64>)`
Creates a new RocksDBSnapshot instance.

```php
<?php
$snapshot = new \RocksDBSnapshot("/path/to/db", 3600); // 3600 seconds TTL
?>
```

#### `create()`
Creates a snapshot of the current state of the database.

```php
<?php
$snapshot->create();
?>
```

#### `release()`
Releases the current snapshot.

```php
<?php
$snapshot->release();
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

Before creating a new instance of any class (e.g., `RocksDB`, `RocksDBIterator`, `RocksDBBackup`, `RocksDBWriteBatch`, `RocksDBSnapshot`, `RocksDBTransaction`), ensure to destroy the previous instance to free up the database connection.

```php
<?php
$db = new \RocksDB($dbPath);
$db = null; // Free the connection

$iterator = new \RocksDBIterator($dbPath); // Now you can create a new instance
?>
```

## Contributing

Contributions are welcome! Please submit pull requests or issues on the [GitHub repository](https://github.com/yourusername/php-rocksdb-rc).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

This project uses the following open-source libraries:
- [ext-php-rs](https://github.com/davidcole1340/ext-php-rs)
- [rust-rocksdb](https://github.com/rust-rocksdb/rust-rocksdb)