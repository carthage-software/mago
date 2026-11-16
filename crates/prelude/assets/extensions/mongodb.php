<?php

namespace {
    /** @var non-empty-string */
    const MONGODB_VERSION = '2.5.7';

    /** @var non-empty-string */
    const MONGODB_STABILITY = 'stable';
}

namespace MongoDB\BSON {
    interface Type {}

    interface Serializable extends Type
    {
        public function bsonSerialize(): array|\stdClass|Document|PackedArray;
    }

    interface Unserializable
    {
        public function bsonUnserialize(array $data): void;
    }

    interface Persistable extends Serializable, Unserializable
    {
        public function bsonSerialize(): array|\stdClass|Document;
    }

    interface BinaryInterface
    {
        public function getData(): string;

        public function getType(): int;

        public function __toString(): string;
    }

    interface Decimal128Interface
    {
        public function __toString(): string;
    }

    interface JavascriptInterface
    {
        public function getCode(): string;

        public function getScope(): ?object;

        public function __toString(): string;
    }

    interface MaxKeyInterface {}

    interface MinKeyInterface {}

    interface ObjectIdInterface
    {
        public function getTimestamp(): int;

        public function __toString(): string;
    }

    interface RegexInterface
    {
        public function getPattern(): string;

        public function getFlags(): string;

        public function __toString(): string;
    }

    interface TimestampInterface
    {
        public function getTimestamp(): int;

        public function getIncrement(): int;

        public function __toString(): string;
    }

    interface UTCDateTimeInterface
    {
        public function toDateTime(): \DateTime;

        public function toDateTimeImmutable(): \DateTimeImmutable;

        public function __toString(): string;
    }

    enum VectorType
    {
        case Float32;
        case Int8;
        case PackedBit;
    }

    final class Binary implements BinaryInterface, \JsonSerializable, Type, \Stringable
    {
        public const TYPE_GENERIC = 0;
        public const TYPE_FUNCTION = 1;
        public const TYPE_OLD_BINARY = 2;
        public const TYPE_OLD_UUID = 3;
        public const TYPE_UUID = 4;
        public const TYPE_MD5 = 5;
        public const TYPE_ENCRYPTED = 6;
        public const TYPE_COLUMN = 7;
        public const TYPE_SENSITIVE = 8;
        public const TYPE_VECTOR = 9;
        public const TYPE_USER_DEFINED = 128;

        /**
         * @param int<0, 255> $type
         *
         * @throws \MongoDB\Driver\Exception\InvalidArgumentException If the data does not fit the type.
         */
        final public function __construct(string $data, int $type = Binary::TYPE_GENERIC) {}

        /**
         * @param list<float>|list<int<-128, 127>>|list<0|1|bool> $vector
         *
         * @throws \MongoDB\Driver\Exception\InvalidArgumentException If the vector does not match the vector type.
         */
        final public static function fromVector(array $vector, VectorType $vectorType): Binary {}

        final public function getData(): string {}

        /**
         * @return int<0, 255>
         */
        final public function getType(): int {}

        /**
         * @throws \MongoDB\Driver\Exception\LogicException If the binary is not a vector.
         */
        final public function getVectorType(): VectorType {}

        /**
         * @return list<float>|list<int<-128, 127>>
         *
         * @throws \MongoDB\Driver\Exception\LogicException If the binary is not a vector.
         */
        final public function toArray(): array {}

        final public static function __set_state(array $properties): Binary {}

        final public function __toString(): string {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$binary': string, '$type': non-empty-string}
         */
        final public function jsonSerialize(): mixed {}
    }

    final class DBPointer implements \JsonSerializable, Type, \Stringable
    {
        final private function __construct() {}

        final public static function __set_state(array $properties): DBPointer {}

        final public function __toString(): string {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$dbPointer': array{'$ref': string, '$id': array{'$oid': non-empty-lowercase-string}}}
         */
        final public function jsonSerialize(): mixed {}
    }

    final class Decimal128 implements Decimal128Interface, \JsonSerializable, Type, \Stringable
    {
        /**
         * @throws \MongoDB\Driver\Exception\InvalidArgumentException If the value is not a valid decimal string.
         */
        final public function __construct(string $value) {}

        /**
         * @return non-empty-string
         */
        final public function __toString(): string {}

        final public static function __set_state(array $properties): Decimal128 {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$numberDecimal': non-empty-string}
         */
        final public function jsonSerialize(): mixed {}
    }

    /**
     * @implements \IteratorAggregate<string, mixed>
     * @implements \ArrayAccess<array-key, mixed>
     */
    final class Document implements \IteratorAggregate, \ArrayAccess, Type, \Stringable
    {
        private function __construct() {}

        /**
         * @throws \MongoDB\Driver\Exception\UnexpectedValueException If the string is not exactly one BSON document.
         */
        final public static function fromBSON(string $bson): Document {}

        /**
         * @throws \MongoDB\Driver\Exception\UnexpectedValueException If the JSON is not a document.
         */
        final public static function fromJSON(string $json): Document {}

        /**
         * @throws \MongoDB\Driver\Exception\UnexpectedValueException If the value cannot be converted to BSON.
         */
        final public static function fromPHP(array|object $value): Document {}

        /**
         * @throws \MongoDB\Driver\Exception\RuntimeException If the key is not in the document.
         */
        final public function get(string $key): mixed {}

        /**
         * @return Iterator<string>
         *
         * @throws \MongoDB\Driver\Exception\UnexpectedValueException If the BSON iterator cannot be initialized.
         */
        final public function getIterator(): Iterator {}

        final public function has(string $key): bool {}

        /**
         * @return ($typeMap is null ? object : array|object)
         *
         * @throws \MongoDB\Driver\Exception\InvalidArgumentException If a class in the type map cannot be used.
         */
        final public function toPHP(?array $typeMap = null): array|object {}

        final public function toCanonicalExtendedJSON(): string {}

        final public function toRelaxedExtendedJSON(): string {}

        public function offsetExists(mixed $offset): bool {}

        /**
         * @throws \MongoDB\Driver\Exception\RuntimeException If the key is not in the document.
         */
        public function offsetGet(mixed $offset): mixed {}

        /**
         * @return never
         *
         * @throws \MongoDB\Driver\Exception\LogicException Always, as a document cannot be modified.
         */
        public function offsetSet(mixed $offset, mixed $value): void {}

        /**
         * @return never
         *
         * @throws \MongoDB\Driver\Exception\LogicException Always, as a document cannot be modified.
         */
        public function offsetUnset(mixed $offset): void {}

        final public function __toString(): string {}

        final public static function __set_state(array $properties): Document {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}
    }

    final class Int64 implements \JsonSerializable, Type, \Stringable
    {
        /**
         * @param int|numeric-string $value
         *
         * @throws \MongoDB\Driver\Exception\InvalidArgumentException If the string is not a 64-bit integer.
         */
        final public function __construct(int|string $value) {}

        /**
         * @return numeric-string
         */
        final public function __toString(): string {}

        final public static function __set_state(array $properties): Int64 {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$numberLong': numeric-string}
         */
        final public function jsonSerialize(): mixed {}
    }

    /**
     * @template-covariant TKey of string|int
     *
     * @implements \Iterator<TKey, mixed>
     */
    final class Iterator implements \Iterator
    {
        final private function __construct() {}

        /**
         * @throws \MongoDB\Driver\Exception\LogicException If the iterator is not valid.
         */
        final public function current(): mixed {}

        /**
         * @return TKey
         *
         * @throws \MongoDB\Driver\Exception\LogicException If the iterator is not valid.
         */
        final public function key(): string|int {}

        final public function next(): void {}

        final public function rewind(): void {}

        final public function valid(): bool {}
    }

    final class Javascript implements JavascriptInterface, \JsonSerializable, Type, \Stringable
    {
        /**
         * @throws \MongoDB\Driver\Exception\InvalidArgumentException If the code contains null bytes.
         */
        final public function __construct(string $code, array|object|null $scope = null) {}

        final public static function __set_state(array $properties): Javascript {}

        final public function getCode(): string {}

        final public function getScope(): ?object {}

        final public function __toString(): string {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$code': string, '$scope'?: object}
         */
        final public function jsonSerialize(): mixed {}
    }

    final class MaxKey implements MaxKeyInterface, \JsonSerializable, Type
    {
        final public static function __set_state(array $properties): MaxKey {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$maxKey': 1}
         */
        final public function jsonSerialize(): mixed {}
    }

    final class MinKey implements MinKeyInterface, \JsonSerializable, Type
    {
        final public static function __set_state(array $properties): MinKey {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$minKey': 1}
         */
        final public function jsonSerialize(): mixed {}
    }

    final class ObjectId implements ObjectIdInterface, \JsonSerializable, Type, \Stringable
    {
        /**
         * @throws \MongoDB\Driver\Exception\InvalidArgumentException If the id is not a 24-character hexadecimal string.
         */
        final public function __construct(?string $id = null) {}

        final public function getTimestamp(): int {}

        /**
         * @return non-empty-lowercase-string
         */
        final public function __toString(): string {}

        final public static function __set_state(array $properties): ObjectId {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$oid': non-empty-lowercase-string}
         */
        final public function jsonSerialize(): mixed {}
    }

    /**
     * @implements \IteratorAggregate<int, mixed>
     * @implements \ArrayAccess<int, mixed>
     */
    final class PackedArray implements \IteratorAggregate, \ArrayAccess, Type, \Stringable
    {
        private function __construct() {}

        /**
         * @throws \MongoDB\Driver\Exception\UnexpectedValueException If the JSON is not an array.
         */
        final public static function fromJSON(string $json): PackedArray {}

        /**
         * @param list<mixed> $value
         *
         * @throws \MongoDB\Driver\Exception\InvalidArgumentException If the array is not a list.
         */
        final public static function fromPHP(array $value): PackedArray {}

        /**
         * @throws \MongoDB\Driver\Exception\RuntimeException If the index is not in the array.
         */
        final public function get(int $index): mixed {}

        /**
         * @return Iterator<int>
         *
         * @throws \MongoDB\Driver\Exception\UnexpectedValueException If the BSON iterator cannot be initialized.
         */
        final public function getIterator(): Iterator {}

        final public function has(int $index): bool {}

        /**
         * @return ($typeMap is null ? list<mixed> : array|object)
         *
         * @throws \MongoDB\Driver\Exception\InvalidArgumentException If a class in the type map cannot be used.
         */
        final public function toPHP(?array $typeMap = null): array|object {}

        final public function toCanonicalExtendedJSON(): string {}

        final public function toRelaxedExtendedJSON(): string {}

        public function offsetExists(mixed $offset): bool {}

        /**
         * @throws \MongoDB\Driver\Exception\RuntimeException If the index is not in the array.
         */
        public function offsetGet(mixed $offset): mixed {}

        /**
         * @return never
         *
         * @throws \MongoDB\Driver\Exception\LogicException Always, as an array cannot be modified.
         */
        public function offsetSet(mixed $offset, mixed $value): void {}

        /**
         * @return never
         *
         * @throws \MongoDB\Driver\Exception\LogicException Always, as an array cannot be modified.
         */
        public function offsetUnset(mixed $offset): void {}

        final public function __toString(): string {}

        final public static function __set_state(array $properties): PackedArray {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}
    }

    final class Regex implements RegexInterface, \JsonSerializable, Type, \Stringable
    {
        /**
         * @throws \MongoDB\Driver\Exception\InvalidArgumentException If the pattern or the flags contain null bytes.
         */
        final public function __construct(string $pattern, string $flags = '') {}

        final public function getPattern(): string {}

        final public function getFlags(): string {}

        /**
         * @return non-empty-string
         */
        final public function __toString(): string {}

        final public static function __set_state(array $properties): Regex {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$regex': string, '$options': string}
         */
        final public function jsonSerialize(): mixed {}
    }

    final class Symbol implements \JsonSerializable, Type, \Stringable
    {
        final private function __construct() {}

        final public function __toString(): string {}

        final public static function __set_state(array $properties): Symbol {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$symbol': string}
         */
        final public function jsonSerialize(): mixed {}
    }

    final class Timestamp implements TimestampInterface, \JsonSerializable, Type, \Stringable
    {
        /**
         * @param int<0, 4294967295>|numeric-string $increment
         * @param int<0, 4294967295>|numeric-string $timestamp
         *
         * @throws \MongoDB\Driver\Exception\InvalidArgumentException If a value is not an unsigned 32-bit integer.
         */
        final public function __construct(int|string $increment, int|string $timestamp) {}

        /**
         * @return int<0, 4294967295>
         */
        final public function getTimestamp(): int {}

        /**
         * @return int<0, 4294967295>
         */
        final public function getIncrement(): int {}

        /**
         * @return non-empty-string
         */
        final public function __toString(): string {}

        final public static function __set_state(array $properties): Timestamp {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$timestamp': array{t: int<0, 4294967295>, i: int<0, 4294967295>}}
         */
        final public function jsonSerialize(): mixed {}
    }

    final class UTCDateTime implements UTCDateTimeInterface, \JsonSerializable, Type, \Stringable
    {
        final public function __construct(int|\DateTimeInterface|Int64|null $milliseconds = null) {}

        final public function toDateTime(): \DateTime {}

        final public function toDateTimeImmutable(): \DateTimeImmutable {}

        /**
         * @return numeric-string
         */
        final public function __toString(): string {}

        final public static function __set_state(array $properties): UTCDateTime {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$date': array{'$numberLong': numeric-string}}
         */
        final public function jsonSerialize(): mixed {}
    }

    final class Undefined implements \JsonSerializable, Type, \Stringable
    {
        final private function __construct() {}

        final public function __toString(): string {}

        final public static function __set_state(array $properties): Undefined {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}

        /**
         * @return array{'$undefined': true}
         */
        final public function jsonSerialize(): mixed {}
    }
}

namespace MongoDB\Driver {
    final class BulkWrite implements \Countable
    {
        final public function __construct(?array $options = null) {}

        /**
         * @return int<0, max>
         */
        public function count(): int {}

        public function delete(array|object $filter, ?array $deleteOptions = null): void {}

        /**
         * @return mixed The `_id` of the inserted document.
         */
        final public function insert(array|object $document): mixed {}

        public function update(array|object $filter, array|object $newObj, ?array $updateOptions = null): void {}
    }

    final class BulkWriteCommand implements \Countable
    {
        final public function __construct(?array $options = null) {}

        /**
         * @return int<0, max>
         */
        public function count(): int {}

        final public function deleteOne(string $namespace, array|object $filter, ?array $options = null): void {}

        final public function deleteMany(string $namespace, array|object $filter, ?array $options = null): void {}

        /**
         * @return mixed The `_id` of the inserted document.
         */
        final public function insertOne(string $namespace, array|object $document): mixed {}

        final public function replaceOne(
            string $namespace,
            array|object $filter,
            array|object $replacement,
            ?array $options = null,
        ): void {}

        final public function updateOne(
            string $namespace,
            array|object $filter,
            array|object $update,
            ?array $options = null,
        ): void {}

        final public function updateMany(
            string $namespace,
            array|object $filter,
            array|object $update,
            ?array $options = null,
        ): void {}
    }

    final class BulkWriteCommandResult
    {
        final private function __construct() {}

        /**
         * @return int<0, max>
         *
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getInsertedCount(): int {}

        /**
         * @return int<0, max>
         *
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getMatchedCount(): int {}

        /**
         * @return int<0, max>
         *
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getModifiedCount(): int {}

        /**
         * @return int<0, max>
         *
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getUpsertedCount(): int {}

        /**
         * @return int<0, max>
         *
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getDeletedCount(): int {}

        /**
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getInsertResults(): ?\MongoDB\BSON\Document {}

        /**
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getUpdateResults(): ?\MongoDB\BSON\Document {}

        /**
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getDeleteResults(): ?\MongoDB\BSON\Document {}

        final public function isAcknowledged(): bool {}
    }

    final class ClientEncryption
    {
        public const AEAD_AES_256_CBC_HMAC_SHA_512_DETERMINISTIC = 'AEAD_AES_256_CBC_HMAC_SHA_512-Deterministic';
        public const AEAD_AES_256_CBC_HMAC_SHA_512_RANDOM = 'AEAD_AES_256_CBC_HMAC_SHA_512-Random';
        public const ALGORITHM_INDEXED = 'Indexed';
        public const ALGORITHM_UNINDEXED = 'Unindexed';
        public const ALGORITHM_RANGE = 'Range';
        public const QUERY_TYPE_EQUALITY = 'equality';
        public const QUERY_TYPE_RANGE = 'range';

        /**
         * @throws Exception\InvalidArgumentException If the options are invalid.
         * @throws Exception\RuntimeException If the extension was built without libmongocrypt.
         */
        final public function __construct(array $options) {}

        /**
         * @throws Exception\RuntimeException
         */
        final public function addKeyAltName(\MongoDB\BSON\Binary $keyId, string $keyAltName): ?object {}

        /**
         * @throws Exception\RuntimeException
         */
        final public function createDataKey(string $kmsProvider, ?array $options = null): \MongoDB\BSON\Binary {}

        /**
         * @throws Exception\EncryptionException If the value cannot be decrypted.
         */
        final public function decrypt(\MongoDB\BSON\Binary $value): mixed {}

        /**
         * @throws Exception\RuntimeException
         */
        final public function deleteKey(\MongoDB\BSON\Binary $keyId): object {}

        /**
         * @throws Exception\EncryptionException If the value cannot be encrypted.
         */
        final public function encrypt(mixed $value, ?array $options = null): \MongoDB\BSON\Binary {}

        /**
         * @throws Exception\EncryptionException If the expression cannot be encrypted.
         */
        final public function encryptExpression(array|object $expr, ?array $options = null): object {}

        /**
         * @throws Exception\RuntimeException
         */
        final public function getKey(\MongoDB\BSON\Binary $keyId): ?object {}

        /**
         * @throws Exception\RuntimeException
         */
        final public function getKeyByAltName(string $keyAltName): ?object {}

        /**
         * @return Cursor<array|object>
         *
         * @throws Exception\RuntimeException
         */
        final public function getKeys(): Cursor {}

        /**
         * @throws Exception\RuntimeException
         */
        final public function removeKeyAltName(\MongoDB\BSON\Binary $keyId, string $keyAltName): ?object {}

        /**
         * @throws Exception\RuntimeException
         */
        final public function rewrapManyDataKey(array|object $filter, ?array $options = null): object {}
    }

    final class Command
    {
        final public function __construct(array|object $document, ?array $commandOptions = null) {}
    }

    /**
     * @template-covariant TValue of array|object
     *
     * @extends \Iterator<int, TValue>
     */
    interface CursorInterface extends \Iterator
    {
        /**
         * @return TValue|null
         *
         * @ignore-nullable-return
         */
        public function current(): array|object|null;

        public function getId(): \MongoDB\BSON\Int64;

        public function getServer(): Server;

        public function isDead(): bool;

        /**
         * @return int<0, max>|null
         *
         * @ignore-nullable-return
         */
        public function key(): ?int;

        public function setTypeMap(array $typemap): void;

        /**
         * @return list<TValue>
         */
        public function toArray(): array;
    }

    /**
     * @template-covariant TValue of array|object
     *
     * @implements CursorInterface<TValue>
     */
    final class Cursor implements CursorInterface
    {
        final private function __construct() {}

        /**
         * @return TValue|null
         *
         * @ignore-nullable-return
         */
        public function current(): array|object|null {}

        final public function getId(): \MongoDB\BSON\Int64 {}

        final public function getServer(): Server {}

        final public function isDead(): bool {}

        /**
         * @return int<0, max>|null
         *
         * @ignore-nullable-return
         */
        public function key(): ?int {}

        /**
         * @throws Exception\RuntimeException If the next batch cannot be fetched.
         */
        public function next(): void {}

        /**
         * @throws Exception\LogicException If the cursor has advanced past its first element.
         * @throws Exception\RuntimeException If the first batch cannot be fetched.
         */
        public function rewind(): void {}

        /**
         * @throws Exception\InvalidArgumentException If a class in the type map cannot be used.
         */
        final public function setTypeMap(array $typemap): void {}

        /**
         * @return list<TValue>
         *
         * @throws Exception\RuntimeException If a batch cannot be fetched.
         */
        final public function toArray(): array {}

        public function valid(): bool {}
    }

    final class Manager
    {
        /**
         * @throws Exception\InvalidArgumentException If an option is invalid.
         * @throws Exception\RuntimeException If the URI cannot be parsed.
         */
        final public function __construct(?string $uri = null, ?array $uriOptions = null, ?array $driverOptions = null) {}

        /**
         * @throws Exception\InvalidArgumentException If a log subscriber is registered with a manager.
         */
        final public function addSubscriber(Monitoring\Subscriber $subscriber): void {}

        /**
         * @throws Exception\InvalidArgumentException If the options are invalid.
         * @throws Exception\RuntimeException If the extension was built without libmongocrypt.
         */
        final public function createClientEncryption(array $options): ClientEncryption {}

        /**
         * @throws Exception\InvalidArgumentException If the bulk write is empty, was already executed, or an option is invalid.
         * @throws Exception\BulkWriteException If a write fails.
         * @throws Exception\RuntimeException
         */
        final public function executeBulkWrite(string $namespace, BulkWrite $bulk, ?array $options = null): WriteResult {}

        /**
         * @throws Exception\InvalidArgumentException If the bulk write is empty, was already executed, or an option is invalid.
         * @throws Exception\BulkWriteCommandException If a write fails.
         * @throws Exception\RuntimeException
         */
        final public function executeBulkWriteCommand(
            BulkWriteCommand $bulkWriteCommand,
            ?array $options = null,
        ): BulkWriteCommandResult {}

        /**
         * @return CursorInterface<array|object>
         *
         * @throws Exception\InvalidArgumentException If an option is invalid.
         * @throws Exception\RuntimeException
         */
        final public function executeCommand(string $db, Command $command, ?array $options = null): CursorInterface {}

        /**
         * @return CursorInterface<array|object>
         *
         * @throws Exception\InvalidArgumentException If an option is invalid.
         * @throws Exception\RuntimeException
         */
        final public function executeQuery(string $namespace, Query $query, ?array $options = null): CursorInterface {}

        /**
         * @return CursorInterface<array|object>
         *
         * @throws Exception\InvalidArgumentException If an option is invalid.
         * @throws Exception\RuntimeException
         */
        final public function executeReadCommand(string $db, Command $command, ?array $options = null): CursorInterface {}

        /**
         * @return CursorInterface<array|object>
         *
         * @throws Exception\InvalidArgumentException If an option is invalid.
         * @throws Exception\RuntimeException
         */
        final public function executeReadWriteCommand(
            string $db,
            Command $command,
            ?array $options = null,
        ): CursorInterface {}

        /**
         * @return CursorInterface<array|object>
         *
         * @throws Exception\InvalidArgumentException If an option is invalid.
         * @throws Exception\RuntimeException
         */
        final public function executeWriteCommand(string $db, Command $command, ?array $options = null): CursorInterface {}

        final public function getEncryptedFieldsMap(): array|object|null {}

        final public function getReadConcern(): ReadConcern {}

        final public function getReadPreference(): ReadPreference {}

        /**
         * @return list<Server>
         */
        final public function getServers(): array {}

        final public function getWriteConcern(): WriteConcern {}

        final public function removeSubscriber(Monitoring\Subscriber $subscriber): void {}

        /**
         * @throws Exception\RuntimeException If no server matches the read preference.
         */
        final public function selectServer(?ReadPreference $readPreference = null): Server {}

        /**
         * @throws Exception\InvalidArgumentException If the options are invalid.
         * @throws Exception\RuntimeException If the session cannot be created.
         */
        final public function startSession(?array $options = null): Session {}
    }

    final class Query
    {
        final public function __construct(array|object $filter, ?array $queryOptions = null) {}
    }

    final class ReadConcern implements \MongoDB\BSON\Serializable
    {
        public const LINEARIZABLE = 'linearizable';
        public const LOCAL = 'local';
        public const MAJORITY = 'majority';
        public const AVAILABLE = 'available';
        public const SNAPSHOT = 'snapshot';

        public readonly string|null $level;

        final public function __construct(?string $level = null) {}

        final public function getLevel(): ?string {}

        final public function isDefault(): bool {}

        final public static function __set_state(array $properties): ReadConcern {}

        final public function bsonSerialize(): \stdClass {}

        final public function __unserialize(array $data): void {}
    }

    final class ReadPreference implements \MongoDB\BSON\Serializable
    {
        public const PRIMARY = 'primary';
        public const PRIMARY_PREFERRED = 'primaryPreferred';
        public const SECONDARY = 'secondary';
        public const SECONDARY_PREFERRED = 'secondaryPreferred';
        public const NEAREST = 'nearest';
        public const NO_MAX_STALENESS = -1;
        public const SMALLEST_MAX_STALENESS_SECONDS = 90;

        /**
         * @var self::PRIMARY|self::PRIMARY_PREFERRED|self::SECONDARY|self::SECONDARY_PREFERRED|self::NEAREST
         */
        public readonly string $mode;

        public readonly array|null $tags;

        /**
         * @var -1|int<90, 2147483647>
         */
        public readonly int $maxStalenessSeconds;

        /**
         * @deprecated Hedged reads are deprecated in MongoDB 8.0.
         */
        public readonly object|null $hedge;

        /**
         * @throws Exception\InvalidArgumentException If the mode, the tag sets, or an option is invalid.
         */
        final public function __construct(string $mode, ?array $tagSets = null, ?array $options = null) {}

        /**
         * @deprecated Hedged reads are deprecated in MongoDB 8.0.
         */
        final public function getHedge(): ?object {}

        /**
         * @return -1|int<90, 2147483647>
         */
        final public function getMaxStalenessSeconds(): int {}

        /**
         * @return self::PRIMARY|self::PRIMARY_PREFERRED|self::SECONDARY|self::SECONDARY_PREFERRED|self::NEAREST
         */
        final public function getModeString(): string {}

        /**
         * @return list<array>
         */
        final public function getTagSets(): array {}

        final public static function __set_state(array $properties): ReadPreference {}

        final public function bsonSerialize(): \stdClass {}

        final public function __unserialize(array $data): void {}
    }

    final class Server
    {
        public const TYPE_UNKNOWN = 0;
        public const TYPE_STANDALONE = 1;
        public const TYPE_MONGOS = 2;
        public const TYPE_POSSIBLE_PRIMARY = 3;
        public const TYPE_RS_PRIMARY = 4;
        public const TYPE_RS_SECONDARY = 5;
        public const TYPE_RS_ARBITER = 6;
        public const TYPE_RS_OTHER = 7;
        public const TYPE_RS_GHOST = 8;
        public const TYPE_LOAD_BALANCER = 9;

        final private function __construct() {}

        /**
         * @throws Exception\InvalidArgumentException If the bulk write is empty, was already executed, or an option is invalid.
         * @throws Exception\BulkWriteException If a write fails.
         * @throws Exception\RuntimeException
         */
        final public function executeBulkWrite(string $namespace, BulkWrite $bulkWrite, ?array $options = null): WriteResult {}

        /**
         * @throws Exception\InvalidArgumentException If the bulk write is empty, was already executed, or an option is invalid.
         * @throws Exception\BulkWriteCommandException If a write fails.
         * @throws Exception\RuntimeException
         */
        final public function executeBulkWriteCommand(
            BulkWriteCommand $bulkWriteCommand,
            ?array $options = null,
        ): BulkWriteCommandResult {}

        /**
         * @return CursorInterface<array|object>
         *
         * @throws Exception\InvalidArgumentException If an option is invalid.
         * @throws Exception\RuntimeException
         */
        final public function executeCommand(string $db, Command $command, ?array $options = null): CursorInterface {}

        /**
         * @return CursorInterface<array|object>
         *
         * @throws Exception\InvalidArgumentException If an option is invalid.
         * @throws Exception\RuntimeException
         */
        final public function executeQuery(string $namespace, Query $query, ?array $options = null): CursorInterface {}

        /**
         * @return CursorInterface<array|object>
         *
         * @throws Exception\InvalidArgumentException If an option is invalid.
         * @throws Exception\RuntimeException
         */
        final public function executeReadCommand(string $db, Command $command, ?array $options = null): CursorInterface {}

        /**
         * @return CursorInterface<array|object>
         *
         * @throws Exception\InvalidArgumentException If an option is invalid.
         * @throws Exception\RuntimeException
         */
        final public function executeReadWriteCommand(
            string $db,
            Command $command,
            ?array $options = null,
        ): CursorInterface {}

        /**
         * @return CursorInterface<array|object>
         *
         * @throws Exception\InvalidArgumentException If an option is invalid.
         * @throws Exception\RuntimeException
         */
        final public function executeWriteCommand(string $db, Command $command, ?array $options = null): CursorInterface {}

        final public function getHost(): string {}

        /**
         * @return array<string, mixed>
         */
        final public function getInfo(): array {}

        final public function getLatency(): ?int {}

        /**
         * @return int<0, 65535>
         */
        final public function getPort(): int {}

        final public function getServerDescription(): ServerDescription {}

        final public function getTags(): array {}

        /**
         * @return self::TYPE_*
         */
        final public function getType(): int {}

        final public function isArbiter(): bool {}

        final public function isHidden(): bool {}

        final public function isPassive(): bool {}

        final public function isPrimary(): bool {}

        final public function isSecondary(): bool {}
    }

    final class ServerApi implements \MongoDB\BSON\Serializable
    {
        public const V1 = '1';

        final public function __construct(string $version, ?bool $strict = null, ?bool $deprecationErrors = null) {}

        final public static function __set_state(array $properties): ServerApi {}

        final public function bsonSerialize(): \stdClass {}

        final public function __unserialize(array $data): void {}

        final public function __serialize(): array {}
    }

    final class ServerDescription
    {
        public const TYPE_UNKNOWN = 'Unknown';
        public const TYPE_STANDALONE = 'Standalone';
        public const TYPE_MONGOS = 'Mongos';
        public const TYPE_POSSIBLE_PRIMARY = 'PossiblePrimary';
        public const TYPE_RS_PRIMARY = 'RSPrimary';
        public const TYPE_RS_SECONDARY = 'RSSecondary';
        public const TYPE_RS_ARBITER = 'RSArbiter';
        public const TYPE_RS_OTHER = 'RSOther';
        public const TYPE_RS_GHOST = 'RSGhost';
        public const TYPE_LOAD_BALANCER = 'LoadBalancer';

        final private function __construct() {}

        /**
         * @return array<string, mixed>
         */
        final public function getHelloResponse(): array {}

        final public function getHost(): string {}

        final public function getLastUpdateTime(): int {}

        /**
         * @return int<0, 65535>
         */
        final public function getPort(): int {}

        final public function getRoundTripTime(): ?int {}

        /**
         * @return self::TYPE_*
         */
        final public function getType(): string {}
    }

    final class Session
    {
        public const TRANSACTION_NONE = 'none';
        public const TRANSACTION_STARTING = 'starting';
        public const TRANSACTION_IN_PROGRESS = 'in_progress';
        public const TRANSACTION_COMMITTED = 'committed';
        public const TRANSACTION_ABORTED = 'aborted';

        final private function __construct() {}

        /**
         * @throws Exception\LogicException If the session has ended.
         * @throws Exception\RuntimeException If the transaction cannot be aborted.
         */
        final public function abortTransaction(): void {}

        /**
         * @throws Exception\LogicException If the session has ended.
         */
        final public function advanceClusterTime(array|object $clusterTime): void {}

        /**
         * @throws Exception\LogicException If the session has ended.
         */
        final public function advanceOperationTime(\MongoDB\BSON\TimestampInterface $operationTime): void {}

        /**
         * @throws Exception\LogicException If the session has ended.
         * @throws Exception\RuntimeException If the transaction cannot be committed.
         */
        final public function commitTransaction(): void {}

        final public function endSession(): void {}

        /**
         * @throws Exception\LogicException If the session has ended.
         */
        final public function getClusterTime(): ?object {}

        /**
         * @throws Exception\LogicException If the session has ended.
         */
        final public function getLogicalSessionId(): object {}

        /**
         * @throws Exception\LogicException If the session has ended.
         */
        final public function getOperationTime(): ?\MongoDB\BSON\Timestamp {}

        /**
         * @throws Exception\LogicException If the session has ended.
         */
        final public function getServer(): ?Server {}

        /**
         * @return array{
         *     maxCommitTimeMS?: int<1, 4294967295>,
         *     readConcern?: ReadConcern,
         *     readPreference?: ReadPreference,
         *     writeConcern?: WriteConcern,
         * }|null
         *
         * @throws Exception\LogicException If the session has ended.
         */
        final public function getTransactionOptions(): ?array {}

        /**
         * @return self::TRANSACTION_*
         *
         * @throws Exception\LogicException If the session has ended.
         */
        final public function getTransactionState(): string {}

        /**
         * @throws Exception\LogicException If the session has ended.
         */
        final public function isDirty(): bool {}

        /**
         * @throws Exception\LogicException If the session has ended.
         */
        final public function isInTransaction(): bool {}

        /**
         * @throws Exception\InvalidArgumentException If an option is invalid.
         * @throws Exception\LogicException If the session has ended.
         * @throws Exception\RuntimeException If the transaction cannot be started.
         */
        final public function startTransaction(?array $options = null): void {}
    }

    final class TopologyDescription
    {
        public const TYPE_UNKNOWN = 'Unknown';
        public const TYPE_SINGLE = 'Single';
        public const TYPE_SHARDED = 'Sharded';
        public const TYPE_REPLICA_SET_NO_PRIMARY = 'ReplicaSetNoPrimary';
        public const TYPE_REPLICA_SET_WITH_PRIMARY = 'ReplicaSetWithPrimary';
        public const TYPE_LOAD_BALANCED = 'LoadBalanced';

        final private function __construct() {}

        /**
         * @return list<ServerDescription>
         */
        final public function getServers(): array {}

        /**
         * @return self::TYPE_*
         */
        final public function getType(): string {}

        final public function hasReadableServer(?ReadPreference $readPreference = null): bool {}

        final public function hasWritableServer(): bool {}
    }

    final class WriteConcern implements \MongoDB\BSON\Serializable
    {
        public const MAJORITY = 'majority';

        public readonly string|int|null $w;

        public readonly bool|null $j;

        /**
         * @var int<0, max>
         */
        public readonly int $wtimeout;

        /**
         * @param string|int<-3, max> $w
         * @param int<0, max>|null $wtimeout
         *
         * @throws Exception\InvalidArgumentException If the write concern is invalid.
         */
        final public function __construct(string|int $w, ?int $wtimeout = null, ?bool $journal = null) {}

        final public function getJournal(): ?bool {}

        final public function getW(): string|int|null {}

        /**
         * @return int<0, max>
         */
        final public function getWtimeout(): int {}

        final public function isDefault(): bool {}

        final public static function __set_state(array $properties): WriteConcern {}

        final public function bsonSerialize(): \stdClass {}

        final public function __unserialize(array $data): void {}
    }

    final class WriteConcernError
    {
        public readonly string $message;

        public readonly int $code;

        public readonly object|null $info;

        final private function __construct() {}

        final public function getCode(): int {}

        final public function getInfo(): ?object {}

        final public function getMessage(): string {}
    }

    final class WriteError
    {
        public readonly string $message;

        public readonly int $code;

        /**
         * @var int<0, max>
         */
        public readonly int $index;

        public readonly object|null $info;

        final private function __construct() {}

        final public function getCode(): int {}

        /**
         * @return int<0, max>
         */
        final public function getIndex(): int {}

        final public function getInfo(): ?object {}

        final public function getMessage(): string {}
    }

    final class WriteResult
    {
        /**
         * @var int<0, max>|null
         */
        public readonly int|null $insertedCount;

        /**
         * @var int<0, max>|null
         */
        public readonly int|null $matchedCount;

        /**
         * @var int<0, max>|null
         */
        public readonly int|null $modifiedCount;

        /**
         * @var int<0, max>|null
         */
        public readonly int|null $deletedCount;

        /**
         * @var int<0, max>|null
         */
        public readonly int|null $upsertedCount;

        public readonly Server $server;

        /**
         * @var array<int<0, max>, mixed>
         */
        public readonly array $upsertedIds;

        /**
         * @var list<WriteError>
         */
        public readonly array $writeErrors;

        public readonly WriteConcernError|null $writeConcernError;

        public readonly WriteConcern|null $writeConcern;

        /**
         * @var list<object>
         */
        public readonly array $errorReplies;

        final private function __construct() {}

        /**
         * @return int<0, max>
         *
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getInsertedCount(): int {}

        /**
         * @return int<0, max>
         *
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getMatchedCount(): int {}

        /**
         * @return int<0, max>
         *
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getModifiedCount(): int {}

        /**
         * @return int<0, max>
         *
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getDeletedCount(): int {}

        /**
         * @return int<0, max>
         *
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getUpsertedCount(): int {}

        final public function getServer(): Server {}

        /**
         * @return array<int<0, max>, mixed>
         *
         * @throws Exception\LogicException If the write was not acknowledged.
         */
        final public function getUpsertedIds(): array {}

        final public function getWriteConcernError(): ?WriteConcernError {}

        /**
         * @return list<WriteError>
         */
        final public function getWriteErrors(): array {}

        /**
         * @return list<object>
         */
        final public function getErrorReplies(): array {}

        final public function isAcknowledged(): bool {}
    }
}

namespace MongoDB\Driver\Exception {
    interface Exception extends \Throwable {}

    class RuntimeException extends \RuntimeException implements Exception
    {
        /**
         * @var list<string>|null
         */
        protected $errorLabels;

        final public function hasErrorLabel(string $errorLabel): bool {}
    }

    class ConnectionException extends RuntimeException {}

    class AuthenticationException extends ConnectionException {}

    final class ConnectionTimeoutException extends ConnectionException {}

    class EncryptionException extends RuntimeException {}

    class ServerException extends RuntimeException {}

    class BulkWriteException extends ServerException
    {
        public readonly \MongoDB\Driver\WriteResult $writeResult;

        final public function getWriteResult(): \MongoDB\Driver\WriteResult {}
    }

    final class BulkWriteCommandException extends ServerException
    {
        private ?\MongoDB\BSON\Document $errorReply = null;

        private ?\MongoDB\Driver\BulkWriteCommandResult $partialResult = null;

        private array $writeErrors = [];

        private array $writeConcernErrors = [];

        final public function getErrorReply(): ?\MongoDB\BSON\Document {}

        final public function getPartialResult(): ?\MongoDB\Driver\BulkWriteCommandResult {}

        /**
         * @return array<int<0, max>, \MongoDB\Driver\WriteError>
         */
        final public function getWriteErrors(): array {}

        /**
         * @return list<\MongoDB\Driver\WriteConcernError>
         */
        final public function getWriteConcernErrors(): array {}
    }

    class CommandException extends ServerException
    {
        public readonly object $resultDocument;

        final public function getResultDocument(): object {}
    }

    final class ExecutionTimeoutException extends ServerException {}

    class InvalidArgumentException extends \InvalidArgumentException implements Exception {}

    class LogicException extends \LogicException implements Exception {}

    class UnexpectedValueException extends \UnexpectedValueException implements Exception {}
}

namespace MongoDB\Driver\Monitoring {
    interface Subscriber {}

    interface CommandSubscriber extends Subscriber
    {
        public function commandStarted(CommandStartedEvent $event): void;

        public function commandSucceeded(CommandSucceededEvent $event): void;

        public function commandFailed(CommandFailedEvent $event): void;
    }

    interface LogSubscriber extends Subscriber
    {
        public const LEVEL_ERROR = 0;
        public const LEVEL_CRITICAL = 1;
        public const LEVEL_WARNING = 2;
        public const LEVEL_MESSAGE = 3;
        public const LEVEL_INFO = 4;
        public const LEVEL_DEBUG = 5;

        /**
         * @param self::LEVEL_* $level
         */
        public function log(int $level, string $domain, string $message): void;
    }

    interface SDAMSubscriber extends Subscriber
    {
        public function serverChanged(ServerChangedEvent $event): void;

        public function serverClosed(ServerClosedEvent $event): void;

        public function serverOpening(ServerOpeningEvent $event): void;

        public function serverHeartbeatFailed(ServerHeartbeatFailedEvent $event): void;

        public function serverHeartbeatStarted(ServerHeartbeatStartedEvent $event): void;

        public function serverHeartbeatSucceeded(ServerHeartbeatSucceededEvent $event): void;

        public function topologyChanged(TopologyChangedEvent $event): void;

        public function topologyClosed(TopologyClosedEvent $event): void;

        public function topologyOpening(TopologyOpeningEvent $event): void;
    }

    final class CommandFailedEvent
    {
        public readonly string $host;

        /**
         * @var int<0, 65535>
         */
        public readonly int $port;

        public readonly string $commandName;

        public readonly string $databaseName;

        /**
         * @var int<0, max>
         */
        public readonly int $duration;

        public readonly \Exception $error;

        public readonly object $reply;

        /**
         * @var numeric-string
         */
        public readonly string $operationId;

        /**
         * @var numeric-string
         */
        public readonly string $requestId;

        public readonly ?\MongoDB\BSON\ObjectId $serviceId;

        public readonly ?int $serverConnectionId;

        final private function __construct() {}

        final public function getCommandName(): string {}

        final public function getDatabaseName(): string {}

        /**
         * @return int<0, max>
         */
        final public function getDurationMicros(): int {}

        final public function getError(): \Exception {}

        final public function getHost(): string {}

        /**
         * @return numeric-string
         */
        final public function getOperationId(): string {}

        /**
         * @return int<0, 65535>
         */
        final public function getPort(): int {}

        final public function getReply(): object {}

        /**
         * @return numeric-string
         */
        final public function getRequestId(): string {}

        final public function getServiceId(): ?\MongoDB\BSON\ObjectId {}

        final public function getServerConnectionId(): ?int {}
    }

    final class CommandStartedEvent
    {
        public readonly string $host;

        /**
         * @var int<0, 65535>
         */
        public readonly int $port;

        public readonly string $commandName;

        public readonly string $databaseName;

        public readonly object $command;

        /**
         * @var numeric-string
         */
        public readonly string $operationId;

        /**
         * @var numeric-string
         */
        public readonly string $requestId;

        public readonly ?\MongoDB\BSON\ObjectId $serviceId;

        public readonly ?int $serverConnectionId;

        final private function __construct() {}

        final public function getCommand(): object {}

        final public function getCommandName(): string {}

        final public function getDatabaseName(): string {}

        final public function getHost(): string {}

        /**
         * @return numeric-string
         */
        final public function getOperationId(): string {}

        /**
         * @return int<0, 65535>
         */
        final public function getPort(): int {}

        /**
         * @return numeric-string
         */
        final public function getRequestId(): string {}

        final public function getServiceId(): ?\MongoDB\BSON\ObjectId {}

        final public function getServerConnectionId(): ?int {}
    }

    final class CommandSucceededEvent
    {
        public readonly string $host;

        /**
         * @var int<0, 65535>
         */
        public readonly int $port;

        public readonly string $commandName;

        public readonly string $databaseName;

        /**
         * @var int<0, max>
         */
        public readonly int $duration;

        public readonly object $reply;

        /**
         * @var numeric-string
         */
        public readonly string $operationId;

        /**
         * @var numeric-string
         */
        public readonly string $requestId;

        public readonly ?\MongoDB\BSON\ObjectId $serviceId;

        public readonly ?int $serverConnectionId;

        final private function __construct() {}

        final public function getCommandName(): string {}

        final public function getDatabaseName(): string {}

        /**
         * @return int<0, max>
         */
        final public function getDurationMicros(): int {}

        final public function getHost(): string {}

        /**
         * @return numeric-string
         */
        final public function getOperationId(): string {}

        /**
         * @return int<0, 65535>
         */
        final public function getPort(): int {}

        final public function getReply(): object {}

        /**
         * @return numeric-string
         */
        final public function getRequestId(): string {}

        final public function getServiceId(): ?\MongoDB\BSON\ObjectId {}

        final public function getServerConnectionId(): ?int {}
    }

    final class ServerChangedEvent
    {
        public readonly string $host;

        /**
         * @var int<0, 65535>
         */
        public readonly int $port;

        public readonly \MongoDB\BSON\ObjectId $topologyId;

        public readonly \MongoDB\Driver\ServerDescription $newDescription;

        public readonly \MongoDB\Driver\ServerDescription $previousDescription;

        final private function __construct() {}

        /**
         * @return int<0, 65535>
         */
        final public function getPort(): int {}

        final public function getHost(): string {}

        final public function getNewDescription(): \MongoDB\Driver\ServerDescription {}

        final public function getPreviousDescription(): \MongoDB\Driver\ServerDescription {}

        final public function getTopologyId(): \MongoDB\BSON\ObjectId {}
    }

    final class ServerClosedEvent
    {
        public readonly string $host;

        /**
         * @var int<0, 65535>
         */
        public readonly int $port;

        public readonly \MongoDB\BSON\ObjectId $topologyId;

        final private function __construct() {}

        /**
         * @return int<0, 65535>
         */
        final public function getPort(): int {}

        final public function getHost(): string {}

        final public function getTopologyId(): \MongoDB\BSON\ObjectId {}
    }

    final class ServerHeartbeatFailedEvent
    {
        public readonly string $host;

        /**
         * @var int<0, 65535>
         */
        public readonly int $port;

        public readonly bool $awaited;

        /**
         * @var int<0, max>
         */
        public readonly int $duration;

        public readonly \Exception $error;

        final private function __construct() {}

        /**
         * @return int<0, max>
         */
        final public function getDurationMicros(): int {}

        final public function getError(): \Exception {}

        /**
         * @return int<0, 65535>
         */
        final public function getPort(): int {}

        final public function getHost(): string {}

        final public function isAwaited(): bool {}
    }

    final class ServerHeartbeatStartedEvent
    {
        public readonly string $host;

        /**
         * @var int<0, 65535>
         */
        public readonly int $port;

        public readonly bool $awaited;

        final private function __construct() {}

        /**
         * @return int<0, 65535>
         */
        final public function getPort(): int {}

        final public function getHost(): string {}

        final public function isAwaited(): bool {}
    }

    final class ServerHeartbeatSucceededEvent
    {
        public readonly string $host;

        /**
         * @var int<0, 65535>
         */
        public readonly int $port;

        public readonly bool $awaited;

        /**
         * @var int<0, max>
         */
        public readonly int $duration;

        public readonly object $reply;

        final private function __construct() {}

        /**
         * @return int<0, max>
         */
        final public function getDurationMicros(): int {}

        final public function getReply(): object {}

        /**
         * @return int<0, 65535>
         */
        final public function getPort(): int {}

        final public function getHost(): string {}

        final public function isAwaited(): bool {}
    }

    final class ServerOpeningEvent
    {
        public readonly string $host;

        /**
         * @var int<0, 65535>
         */
        public readonly int $port;

        public readonly \MongoDB\BSON\ObjectId $topologyId;

        final private function __construct() {}

        /**
         * @return int<0, 65535>
         */
        final public function getPort(): int {}

        final public function getHost(): string {}

        final public function getTopologyId(): \MongoDB\BSON\ObjectId {}
    }

    final class TopologyChangedEvent
    {
        public readonly \MongoDB\BSON\ObjectId $topologyId;

        public readonly \MongoDB\Driver\TopologyDescription $newDescription;

        public readonly \MongoDB\Driver\TopologyDescription $previousDescription;

        final private function __construct() {}

        final public function getNewDescription(): \MongoDB\Driver\TopologyDescription {}

        final public function getPreviousDescription(): \MongoDB\Driver\TopologyDescription {}

        final public function getTopologyId(): \MongoDB\BSON\ObjectId {}
    }

    final class TopologyClosedEvent
    {
        public readonly \MongoDB\BSON\ObjectId $topologyId;

        final private function __construct() {}

        final public function getTopologyId(): \MongoDB\BSON\ObjectId {}
    }

    final class TopologyOpeningEvent
    {
        public readonly \MongoDB\BSON\ObjectId $topologyId;

        final private function __construct() {}

        final public function getTopologyId(): \MongoDB\BSON\ObjectId {}
    }

    function addSubscriber(Subscriber $subscriber): void {}

    /**
     * @internal
     */
    function mongoc_log(int $level, string $domain, string $message): void {}

    function removeSubscriber(Subscriber $subscriber): void {}
}
