<?php

const LEVELDB_NO_COMPRESSION = 0;

const LEVELDB_SNAPPY_COMPRESSION = 1;

class LevelDB
{
    /**
     * @param string $name Path to database
     */
    public function __construct($name, array $options = [], array $read_options = [], array $write_options = []) {}

    /**
     * @param string $key
     *
     * @return string|false
     */
    public function get($key, array $read_options = []) {}

    /**
     * @param string $key
     * @param string $value
     */
    public function set($key, $value, array $write_options = []) {}

    /**
     * @param string $key
     * @param string $value
     */
    public function put($key, $value, array $write_options = []) {}

    /**
     * @param string $key
     *
     * @return bool
     */
    public function delete($key, array $write_options = []) {}

    public function write(LevelDBWriteBatch $batch, array $write_options = []) {}

    /**
     * @param string $name
     */
    public function getProperty($name) {}

    public function getApproximateSizes($start, $limit) {}

    public function compactRange($start, $limit) {}

    public function close() {}

    /**
     * @return LevelDBIterator
     */
    public function getIterator(array $options = []) {}

    /**
     * @return LevelDBSnapshot
     */
    public function getSnapshot() {}

    public static function destroy($name, array $options = []) {}

    public static function repair($name, array $options = []) {}
}

class LevelDBIterator implements Iterator
{
    public function __construct(LevelDB $db, array $read_options = []) {}

    public function valid() {}

    public function rewind() {}

    public function last() {}

    public function seek($key) {}

    public function next() {}

    public function prev() {}

    public function key() {}

    public function current() {}

    public function getError() {}

    public function destroy() {}
}

class LevelDBWriteBatch
{
    public function __construct() {}

    public function set($key, $value, array $write_options = []) {}

    public function put($key, $value, array $write_options = []) {}

    public function delete($key, array $write_options = []) {}

    public function clear() {}
}

class LevelDBSnapshot
{
    public function __construct(LevelDB $db) {}

    public function release() {}
}

class LevelDBException extends Exception {}
