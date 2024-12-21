<?php

namespace MongoDB\BSON;

/**
 * @since 1.16.0
 * @link https://secure.php.net/manual/en/class.mongodb-bson-packedarray.php
 */
final class PackedArray implements \ArrayAccess, \IteratorAggregate, \Serializable
{
    private function __construct() {}

    /** @since 1.20.0 */
    final public static function fromJSON(string $json): PackedArray {}

    final public static function fromPHP(array $value): PackedArray {}

    final public function get(int $index): mixed {}

    final public function getIterator(): Iterator {}

    final public function has(int $index): bool {}

    final public function toPHP(?array $typeMap = null): array|object {}

    /** @since 1.20.0 */
    final public function toCanonicalExtendedJSON(): string {}

    /** @since 1.20.0 */
    final public function toRelaxedExtendedJSON(): string {}

    /** @since 1.17.0 */
    public function offsetExists(mixed $offset): bool {}

    /** @since 1.17.0 */
    public function offsetGet(mixed $offset): mixed {}

    /** @since 1.17.0 */
    public function offsetSet(mixed $offset, mixed $value): void {}

    /** @since 1.17.0 */
    public function offsetUnset(mixed $offset): void {}

    final public function __toString(): string {}

    final public static function __set_state(array $properties): PackedArray {}

    final public function serialize(): string {}

    final public function unserialize(string $data): void {}

    final public function __unserialize(array $data): void {}

    final public function __serialize(): array {}
}
