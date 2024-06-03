<?php

/**
 * A PHP extension for interacting with RocksDB, built with Rust.
 */
class RocksDB {
    /**
     * Creates a new RocksDB instance with the specified path and TTL.
     * @param string $path
     * @param int|null $ttl_secs
     */
    public function __construct(string $path, ?int $ttl_secs = null) {}

    /**
     * Inserts a key-value pair into the database.
     * @param string $key
     * @param string $value
     * @param string|null $cf_name
     * @return void
     */
    public function put(string $key, string $value, ?string $cf_name = null) {}

    /**
     * Retrieves the value associated with the given key.
     * @param string $key
     * @param string|null $cf_name
     * @return string|null
     */
    public function get(string $key, ?string $cf_name = null): ?string {}

    /**
     * Merges a value into the database.
     * @param string $key
     * @param string $value
     * @param string|null $cf_name
     * @return void
     */
    public function merge(string $key, string $value, ?string $cf_name = null) {}

    /**
     * Deletes the key-value pair associated with the given key.
     * @param string $key
     * @param string|null $cf_name
     * @return void
     */
    public function delete(string $key, ?string $cf_name = null) {}

    /**
     * Lists all column families in the database.
     * @param string $path
     * @return string[]
     */
    public static function listColumnFamilies(string $path): array {}

    /**
     * Creates a new column family with the specified name.
     * @param string $cf_name
     * @return void
     */
    public function createColumnFamily(string $cf_name) {}

    /**
     * Drops the column family with the specified name.
     * @param string $cf_name
     * @return void
     */
    public function dropColumnFamily(string $cf_name) {}

    /**
     * Retrieves a database property.
     * @param string $property
     * @param string|null $cf_name
     * @return string|null
     */
    public function getProperty(string $property, ?string $cf_name = null): ?string {}

    /**
     * Flushes all memtable data to SST files.
     * @param string|null $cf_name
     * @return void
     */
    public function flush(?string $cf_name = null) {}

    /**
     * Repairs a RocksDB database at the specified path.
     * @param string $path
     * @return void
     */
    public static function repair(string $path) {}

    /**
     * Closes the RocksDB instance.
     * @return void
     */
    public function close() {}

    /**
     * Returns all key-value pairs in the database or column family.
     * @param string|null $cf_name
     * @return array
     */
    public function all(?string $cf_name = null): array {}

    /**
     * Returns all keys in the database or column family.
     * @param string|null $cf_name
     * @return string[]
     */
    public function keys(?string $cf_name = null): array {}


     /**
     * Moves the iterator to the first element.
     * @return void
     */
    public function seekToFirst() {}

    /**
     * Moves the iterator to the last element.
     * @return void
     */
    public function seekToLast() {}

    /**
     * Moves the iterator to the specified key or the nearest key greater than the specified key.
     * @param string $key
     * @return void
     */
    public function seek(string $key) {}

    /**
     * Moves the iterator to the specified key or the nearest key less than or equal to the specified key.
     * @param string $key
     * @return void
     */
    public function seekForPrev(string $key) {}

    /**
     * Checks if the current position of the iterator is valid.
     * @return bool
     */
    public function valid(): bool {}

    /**
     * Moves the iterator to the next element and returns the current key-value pair.
     * @return array|null
     */
    public function next(): ?array {}

    /**
     * Moves the iterator to the previous element and returns the current key-value pair.
     * @return array|null
     */
    public function prev(): ?array {}
}

class RocksDBBackup {
    /**
     * Creates a new RocksDBBackup instance with the specified path and TTL.
     * @param string $path
     * @param int|null $ttl_secs
     */
    public function __construct(string $path, ?int $ttl_secs = null) {}

    /**
     * Initializes the backup engine with the specified path.
     * @param string $backup_path
     * @return void
     */
    public function init(string $backup_path) {}

    /**
     * Creates a backup of the database.
     * @return void
     */
    public function create() {}

    /**
     * Returns information about the backups.
     * @return array
     */
    public function info(): array {}

    /**
     * Purges old backups, keeping the specified number of backups.
     * @param int $num_backups_to_keep
     * @return void
     */
    public function purge_old(int $num_backups_to_keep) {}

    /**
     * Restores the database from a backup.
     * @param int $backup_id
     * @param string $restore_path
     * @return void
     */
    public function restore(int $backup_id, string $restore_path) {}
}

class RocksDBSnapshot {
    /**
     * Creates a new RocksDBSnapshot instance with the specified path and TTL.
     * @param string $path
     * @param int|null $ttl_secs
     */
    public function __construct(string $path, ?int $ttl_secs = null) {}

    /**
     * Creates a snapshot of the current state of the database.
     * @return void
     */
    public function create() {}

    /**
     * Releases the current snapshot.
     * @return void
     */
    public function release() {}
}

class RocksDBTransaction {
    /**
     * Creates a new RocksDBTransaction instance with the specified path and TTL.
     * @param string $path
     * @param int|null $ttl_secs
     */
    public function __construct(string $path, ?int $ttl_secs = null) {}

    /**
     * Starts a new transaction.
     * @return void
     */
    public function start() {}

    /**
     * Commits the current transaction.
     * @return void
     */
    public function commit() {}

    /**
     * Rolls back the current transaction.
     * @return void
     */
    public function rollback() {}

    /**
     * Sets a savepoint within the current transaction.
     * @return void
     */
    public function set_savepoint() {}

    /**
     * Rolls back the transaction to the last savepoint.
     * @return void
     */
    public function rollback_to_savepoint() {}

    /**
     * Puts a key-value pair into the current transaction.
     * @param string $key
     * @param string $value
     * @param string|null $cf_name
     * @return void
     */
    public function put(string $key, string $value, ?string $cf_name = null) {}

    /**
     * Gets the value associated with the given key within the current transaction.
     * @param string $key
     * @param string|null $cf_name
     * @return string|null
     */
    public function get(string $key, ?string $cf_name = null): ?string {}

    /**
     * Deletes a key-value pair within the current transaction.
     * @param string $key
     * @param string|null $cf_name
     * @return void
     */
    public function delete(string $key, ?string $cf_name = null) {}

    /**
     * Merges a value within the current transaction.
     * @param string $key
     * @param string $value
     * @param string|null $cf_name
     * @return void
     */
    public function merge(string $key, string $value, ?string $cf_name = null) {}
}
