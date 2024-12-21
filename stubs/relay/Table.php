<?php

namespace Relay;

/**
 * Relay Table is a persistent per-worker hash table that can store arbitrary data.
 */
class Table
{
    /**
     * Create a Relay table instance.
     *
     * @param  string|null  $namespace
     * @param  int  $serializer
     */
    public function __construct(?string $namespace = null, int $serializer = Relay::SERIALIZER_PHP) {}

    /**
     * Returns a key, or `null` if key doesn't exist.
     *
     * @param  string  $key
     * @return mixed
     */
    public function get(string $key): mixed {}

    /**
     * Pluck a key from a cached key.
     *
     * @param  string  $key
     * @param  string  $field
     * @return mixed
     */
    public function pluck(string $key, string $field): mixed {}

    /**
     * Set a key and its value.
     *
     * @param  string  $key
     * @param  mixed  $value {}
     * @return bool
     */
    public function set(string $key, mixed $value): bool {}

    /**
     * Check if a key exists in the table.
     *
     * @param  string  $key
     * @return bool
     */
    public function exists(string $key): bool {}

    /**
     * Remove a key from the table.
     *
     * @param  string  $key
     * @return bool
     */
    public function delete(string $key): bool {}

    /**
     * The number of keys stored in the table.
     *
     * @return int
     */
    public function count(): int {}

    /**
     * Get the table's namespace.
     *
     * @return string|null
     */
    public function namespace(): string|null {}

    /**
     * Returns all table namespaces.
     *
     * @return array
     */
    public static function namespaces(): array {}

    /**
     * Removes all keys from the table.
     *
     * @return bool
     */
    public function clear(): bool {}

    /**
     * Removes all keys from all tables.
     *
     * @return int
     */
    public static function clearAll(): int {}
}
