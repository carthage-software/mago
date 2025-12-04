<?php

const MONGODB_VERSION = '1.16.0';

const MONGODB_STABILITY = 'stable';

namespace MongoDB\BSON {
    interface Type
    {
    }

    interface Serializable extends Type
    {
        public function bsonSerialize(): array|\stdClass;
    }

    interface Unserializable
    {
        public function bsonUnserialize(array $data): void;
    }

    interface Persistable extends Serializable, Unserializable
    {
    }

    interface ObjectIdInterface
    {
        public function getTimestamp(): int;

        public function __toString(): string;
    }

    interface BinaryInterface
    {
        public function getData(): string;

        public function getType(): int;

        public function __toString(): string;
    }

    interface UTCDateTimeInterface
    {
        public function toDateTime(): \DateTime;

        public function __toString(): string;
    }

    interface TimestampInterface
    {
        public function getIncrement(): int;

        public function getTimestamp(): int;

        public function __toString(): string;
    }

    interface RegexInterface
    {
        public function getFlags(): string;

        public function getPattern(): string;

        public function __toString(): string;
    }

    interface Decimal128Interface
    {
        public function __toString(): string;
    }

    interface MaxKeyInterface
    {
    }

    interface MinKeyInterface
    {
    }

    /**
     * @implements \IteratorAggregate<string, mixed>
     * @implements \ArrayAccess<string, mixed>
     */
    final class Document implements \IteratorAggregate, \ArrayAccess, Type, \Stringable
    {
        private function __construct() {}

        final public static function fromBSON(string $bson): Document
        {
        }

        final public static function fromJSON(string $json): Document
        {
        }

        final public static function fromPHP(array|object $value): Document
        {
        }

        final public function get(string $key): mixed
        {
        }

        final public function getIterator(): Iterator
        {
        }

        final public function has(string $key): bool
        {
        }

        final public function toPHP(null|array $typeMap = null): array|object
        {
        }

        final public function toCanonicalExtendedJSON(): string
        {
        }

        final public function toRelaxedExtendedJSON(): string
        {
        }

        public function offsetExists(mixed $offset): bool
        {
        }

        public function offsetGet(mixed $offset): mixed
        {
        }

        public function offsetSet(mixed $offset, mixed $value): void
        {
        }

        public function offsetUnset(mixed $offset): void
        {
        }

        final public function __toString(): string
        {
        }

        final public static function __set_state(array $properties): Document
        {
        }

        final public function __unserialize(array $data): void
        {
        }

        final public function __serialize(): array
        {
        }
    }

    /**
     * @implements \IteratorAggregate<int, mixed>
     * @implements \ArrayAccess<int, mixed>
     */
    final class PackedArray implements \IteratorAggregate, \ArrayAccess, Type, \Stringable
    {
        private function __construct() {}

        final public static function fromPHP(array $value): PackedArray
        {
        }

        final public function get(int $key): mixed
        {
        }

        final public function getIterator(): Iterator
        {
        }

        final public function has(int $key): bool
        {
        }

        final public function toPHP(null|array $typeMap = null): array
        {
        }

        final public function toCanonicalExtendedJSON(): string
        {
        }

        final public function toRelaxedExtendedJSON(): string
        {
        }

        public function offsetExists(mixed $offset): bool
        {
        }

        public function offsetGet(mixed $offset): mixed
        {
        }

        public function offsetSet(mixed $offset, mixed $value): void
        {
        }

        public function offsetUnset(mixed $offset): void
        {
        }

        final public function __toString(): string
        {
        }

        final public static function __set_state(array $properties): PackedArray
        {
        }

        final public function __unserialize(array $data): void
        {
        }

        final public function __serialize(): array
        {
        }
    }

    /**
     * @implements \Iterator<string, mixed>
     */
    final class Iterator implements \Iterator
    {
        final private function __construct() {}

        public function current(): mixed
        {
        }

        public function key(): null|string
        {
        }

        public function next(): void
        {
        }

        public function rewind(): void
        {
        }

        public function valid(): bool
        {
        }
    }

    final class ObjectId implements Type, ObjectIdInterface, \JsonSerializable, \Stringable
    {
        final public function __construct(null|string $id = null) {}

        final public function __toString(): string
        {
        }

        public static function __set_state(array $properties): self
        {
        }

        final public function getTimestamp(): int
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }

    final class Binary implements Type, BinaryInterface, \JsonSerializable, \Stringable
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
        public const TYPE_USER_DEFINED = 128;

        final public function __construct(string $data, int $type = Binary::TYPE_GENERIC) {}

        final public function getData(): string
        {
        }

        final public function getType(): int
        {
        }

        public static function __set_state(array $properties): self
        {
        }

        final public function __toString(): string
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }

    final class UTCDateTime implements UTCDateTimeInterface, \JsonSerializable, Type, \Stringable
    {
        final public function __construct(int|\DateTimeInterface|Int64|null $milliseconds = null) {}

        public static function __set_state(array $properties): self
        {
        }

        final public function toDateTime(): \DateTime
        {
        }

        final public function toDateTimeImmutable(): \DateTimeImmutable
        {
        }

        final public function __toString(): string
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }

    final class Timestamp implements TimestampInterface, \JsonSerializable, Type, \Stringable
    {
        final public function __construct(int $increment, int $timestamp) {}

        final public function __toString(): string
        {
        }

        public static function __set_state(array $properties): self
        {
        }

        final public function getIncrement(): int
        {
        }

        final public function getTimestamp(): int
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }

    final class Regex implements RegexInterface, \JsonSerializable, Type, \Stringable
    {
        final public function __construct(string $pattern, string $flags = '') {}

        final public function getFlags(): string
        {
        }

        final public function getPattern(): string
        {
        }

        final public function __toString(): string
        {
        }

        public static function __set_state(array $properties): self
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }

    final class Decimal128 implements Decimal128Interface, \JsonSerializable, Type, \Stringable
    {
        final public function __construct(string $value) {}

        final public function __toString(): string
        {
        }

        public static function __set_state(array $properties): self
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }

    final class Int64 implements \JsonSerializable, Type, \Stringable
    {
        final public function __construct(int|string $value) {}

        final public function __toString(): string
        {
        }

        public static function __set_state(array $properties): self
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }

    final class MaxKey implements MaxKeyInterface, \JsonSerializable, Type
    {
        public static function __set_state(array $properties): self
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }

    final class MinKey implements MinKeyInterface, \JsonSerializable, Type
    {
        public static function __set_state(array $properties): self
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }

    final class Javascript implements \JsonSerializable, Type, \Stringable
    {
        final public function __construct(string $code, array|object|null $scope = null) {}

        public static function __set_state(array $properties): self
        {
        }

        final public function getCode(): string
        {
        }

        final public function getScope(): null|object
        {
        }

        final public function __toString(): string
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }

    final class Symbol implements \JsonSerializable, Type, \Stringable
    {
        final private function __construct() {}

        public static function __set_state(array $properties): self
        {
        }

        final public function __toString(): string
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }

    final class Undefined implements \JsonSerializable, Type, \Stringable
    {
        final private function __construct() {}

        public static function __set_state(array $properties): self
        {
        }

        final public function __toString(): string
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }

    final class DBPointer implements \JsonSerializable, Type, \Stringable
    {
        final private function __construct() {}

        public static function __set_state(array $properties): self
        {
        }

        final public function __toString(): string
        {
        }

        final public function jsonSerialize(): mixed
        {
        }
    }
}

namespace MongoDB\Driver\Exception {
    interface Exception extends \Throwable
    {
    }

    class RuntimeException extends \RuntimeException implements Exception
    {
        protected $errorLabels;

        final public function hasErrorLabel(string $label): bool
        {
        }
    }

    class ConnectionException extends RuntimeException
    {
    }

    class AuthenticationException extends ConnectionException
    {
    }

    class ConnectionTimeoutException extends ConnectionException
    {
    }

    class ServerException extends RuntimeException
    {
    }

    class CommandException extends ServerException
    {
    }

    class ExecutionTimeoutException extends ServerException
    {
    }

    class InvalidArgumentException extends \InvalidArgumentException implements Exception
    {
    }

    class LogicException extends \LogicException implements Exception
    {
    }

    class UnexpectedValueException extends \UnexpectedValueException implements Exception
    {
    }

    class EncryptionException extends RuntimeException
    {
    }

    class BulkWriteException extends RuntimeException
    {
        final public function getWriteResult(): \MongoDB\Driver\WriteResult
        {
        }
    }

    class WriteConcernException extends RuntimeException
    {
        final public function getWriteResult(): \MongoDB\Driver\WriteResult
        {
        }
    }
}

namespace MongoDB\Driver {
    use Iterator;
    use MongoDB\BSON\Int64;
    use MongoDB\BSON\Serializable;
    use MongoDB\Driver\Exception\AuthenticationException;
    use MongoDB\Driver\Exception\BulkWriteException;
    use MongoDB\Driver\Exception\ConnectionException;
    use MongoDB\Driver\Exception\Exception;
    use MongoDB\Driver\Exception\InvalidArgumentException;
    use MongoDB\Driver\Exception\RuntimeException;
    use MongoDB\Driver\Exception\WriteConcernException;
    use MongoDB\Driver\Monitoring\Subscriber;

    /**
     * @implements Iterator<int, array|object>
     */
    interface CursorInterface extends Iterator
    {
        public function current(): array|object|null;

        public function getId(): Int64;

        public function getServer(): Server;

        public function isDead(): bool;

        public function key(): null|int;

        public function setTypeMap(array $typemap): void;

        public function toArray(): array;
    }

    final class Manager
    {
        final public function __construct(
            null|string $uri = null,
            null|array $uriOptions = null,
            null|array $driverOptions = null,
        ) {}

        final public function __wakeup()
        {
        }

        final public function createClientEncryption(array $options)
        {
        }

        final public function executeBulkWrite(
            string $namespace,
            BulkWrite $bulk,
            array|null $options = null,
        ): WriteResult {
        }

        final public function executeCommand(string $db, Command $command, array|null $options = null): CursorInterface
        {
        }

        final public function executeQuery(string $namespace, Query $query, array|null $options = null): CursorInterface
        {
        }

        final public function executeReadCommand(
            string $db,
            Command $command,
            null|array $options = null,
        ): CursorInterface {
        }

        final public function executeReadWriteCommand(
            string $db,
            Command $command,
            null|array $options = null,
        ): CursorInterface {
        }

        final public function executeWriteCommand(
            string $db,
            Command $command,
            null|array $options = null,
        ): CursorInterface {
        }

        final public function getEncryptedFieldsMap(): array|object|null
        {
        }

        final public function getReadConcern(): ReadConcern
        {
        }

        final public function getReadPreference(): ReadPreference
        {
        }

        /** @return Server[] */
        final public function getServers(): array
        {
        }

        final public function getWriteConcern(): WriteConcern
        {
        }

        final public function selectServer(null|ReadPreference $readPreference = null)
        {
        }

        final public function startSession(null|array $options = null)
        {
        }

        final public function addSubscriber(Subscriber $subscriber): void
        {
        }

        final public function removeSubscriber(Subscriber $subscriber): void
        {
        }
    }

    final class Cursor implements CursorInterface
    {
        final private function __construct() {}

        final public function __wakeup()
        {
        }

        public function current(): array|object|null
        {
        }

        final public function getId(): Int64
        {
        }

        final public function getServer(): Server
        {
        }

        final public function isDead(): bool
        {
        }

        public function key(): null|int
        {
        }

        public function next(): void
        {
        }

        public function rewind(): void
        {
        }

        final public function setTypeMap(array $typemap): void
        {
        }

        final public function toArray(): array
        {
        }

        public function valid(): bool
        {
        }
    }

    final class Command
    {
        final public function __construct(array|object $document, null|array $commandOptions = null) {}

        final public function __wakeup()
        {
        }
    }

    final class Query
    {
        final public function __construct(array|object $filter, null|array $queryOptions = null) {}

        final public function __wakeup()
        {
        }
    }

    final class BulkWrite implements \Countable
    {
        final public function __construct(null|array $options = null) {}

        final public function __wakeup()
        {
        }

        final public function count(): int
        {
        }

        final public function delete(array|object $filter, null|array $deleteOptions = null): void
        {
        }

        final public function insert(array|object $document)
        {
        }

        final public function update(array|object $filter, array|object $newObj, null|array $updateOptions = null)
        {
        }
    }

    final class ReadConcern implements Serializable
    {
        public const LINEARIZABLE = 'linearizable';
        public const LOCAL = 'local';
        public const MAJORITY = 'majority';
        public const AVAILABLE = 'available';
        public const SNAPSHOT = 'snapshot';

        final public function __construct(null|string $level = null) {}

        public static function __set_state(array $properties)
        {
        }

        final public function getLevel(): null|string
        {
        }

        final public function bsonSerialize(): \stdClass
        {
        }

        final public function isDefault(): bool
        {
        }
    }

    final class WriteConcern implements Serializable
    {
        public const MAJORITY = 'majority';

        final public function __construct(string|int $w, null|int $wtimeout = null, null|bool $journal = null) {}

        public static function __set_state(array $properties)
        {
        }

        final public function getJournal(): null|bool
        {
        }

        final public function getW(): string|int|null
        {
        }

        final public function getWtimeout(): int
        {
        }

        final public function bsonSerialize(): \stdClass
        {
        }

        final public function isDefault(): bool
        {
        }
    }

    final class ReadPreference implements Serializable
    {
        public const PRIMARY = 'primary';
        public const PRIMARY_PREFERRED = 'primaryPreferred';
        public const SECONDARY = 'secondary';
        public const SECONDARY_PREFERRED = 'secondaryPreferred';
        public const NEAREST = 'nearest';
        public const NO_MAX_STALENESS = -1;
        public const SMALLEST_MAX_STALENESS_SECONDS = 90;

        final public function __construct(string $mode, null|array $tagSets = null, null|array $options = null) {}

        public static function __set_state(array $properties)
        {
        }

        final public function getHedge(): null|object
        {
        }

        final public function getModeString(): string
        {
        }

        final public function getTagSets(): array
        {
        }

        final public function bsonSerialize(): \stdClass
        {
        }

        final public function getMaxStalenessSeconds()
        {
        }
    }

    final class WriteResult
    {
        final private function __construct() {}

        final public function __wakeup()
        {
        }

        final public function getDeletedCount(): int
        {
        }

        final public function getInsertedCount(): int
        {
        }

        final public function getMatchedCount(): int
        {
        }

        final public function getModifiedCount(): int
        {
        }

        final public function getServer(): Server
        {
        }

        final public function getUpsertedCount(): int
        {
        }

        final public function getUpsertedIds(): array
        {
        }

        final public function getWriteConcernError(): null|WriteConcernError
        {
        }

        /** @return WriteError[] */
        final public function getWriteErrors(): array
        {
        }

        final public function getErrorReplies(): array
        {
        }

        final public function isAcknowledged(): bool
        {
        }
    }

    final class WriteError
    {
        final private function __construct() {}

        final public function getCode(): int
        {
        }

        final public function getIndex(): int
        {
        }

        final public function getInfo(): null|object
        {
        }

        final public function getMessage(): string
        {
        }
    }

    final class WriteConcernError
    {
        final private function __construct() {}

        final public function getCode(): int
        {
        }

        final public function getInfo(): null|object
        {
        }

        final public function getMessage(): string
        {
        }
    }

    final class Server
    {
        final private function __construct() {}

        final public function __wakeup()
        {
        }

        final public function executeBulkWrite(
            string $namespace,
            BulkWrite $bulk,
            null|array $options = null,
        ): WriteResult {
        }

        final public function executeCommand(string $db, Command $command, null|array $options = null): CursorInterface
        {
        }

        final public function executeQuery(string $namespace, Query $query, null|array $options = null): CursorInterface
        {
        }

        final public function executeReadCommand(
            string $db,
            Command $command,
            null|array $options = null,
        ): CursorInterface {
        }

        final public function executeReadWriteCommand(
            string $db,
            Command $command,
            null|array $options = null,
        ): CursorInterface {
        }

        final public function executeWriteCommand(
            string $db,
            Command $command,
            null|array $options = null,
        ): CursorInterface {
        }

        final public function getHost(): string
        {
        }

        final public function getInfo(): array
        {
        }

        final public function getLatency(): null|int
        {
        }

        final public function getPort(): int
        {
        }

        final public function getServerDescription(): ServerDescription
        {
        }

        final public function getTags(): array
        {
        }

        final public function getType(): int
        {
        }

        final public function isArbiter(): bool
        {
        }

        final public function isHidden(): bool
        {
        }

        final public function isPassive(): bool
        {
        }

        final public function isPrimary(): bool
        {
        }

        final public function isSecondary(): bool
        {
        }
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

        final public function getHelloResponse(): array
        {
        }

        final public function getHost(): string
        {
        }

        final public function getLastUpdateTime(): int
        {
        }

        final public function getPort(): int
        {
        }

        final public function getRoundTripTime(): null|int
        {
        }

        final public function getType(): string
        {
        }
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

        /** @return ServerDescription[] */
        final public function getServers(): array
        {
        }

        final public function getType(): string
        {
        }

        final public function hasReadableServer(null|ReadPreference $readPreference = null): bool
        {
        }

        final public function hasWritableServer(): bool
        {
        }
    }

    final class Session
    {
        public const TRANSACTION_NONE = 'none';
        public const TRANSACTION_STARTING = 'starting';
        public const TRANSACTION_IN_PROGRESS = 'in_progress';
        public const TRANSACTION_COMMITTED = 'committed';
        public const TRANSACTION_ABORTED = 'aborted';

        final private function __construct() {}

        final public function abortTransaction(): void
        {
        }

        final public function advanceClusterTime(array|object $clusterTime): void
        {
        }

        final public function advanceOperationTime(\MongoDB\BSON\Timestamp $operationTime): void
        {
        }

        final public function commitTransaction(): void
        {
        }

        final public function endSession(): void
        {
        }

        final public function getClusterTime(): null|object
        {
        }

        final public function getLogicalSessionId(): object
        {
        }

        final public function getOperationTime(): null|\MongoDB\BSON\Timestamp
        {
        }

        final public function getServer(): null|Server
        {
        }

        final public function getTransactionOptions(): null|array
        {
        }

        final public function getTransactionState(): string
        {
        }

        final public function isDirty(): bool
        {
        }

        final public function isInTransaction(): bool
        {
        }

        final public function startTransaction(null|array $options = null): void
        {
        }
    }

    final class ClientEncryption
    {
        public const AEAD_AES_256_CBC_HMAC_SHA_512_DETERMINISTIC = 'AEAD_AES_256_CBC_HMAC_SHA_512-Deterministic';
        public const AEAD_AES_256_CBC_HMAC_SHA_512_RANDOM = 'AEAD_AES_256_CBC_HMAC_SHA_512-Random';
        public const ALGORITHM_INDEXED = 'Indexed';
        public const ALGORITHM_UNINDEXED = 'Unindexed';
        public const ALGORITHM_RANGE_PREVIEW = 'RangePreview';
        public const QUERY_TYPE_EQUALITY = 'equality';
        public const QUERY_TYPE_RANGE_PREVIEW = 'rangePreview';

        final public function __construct(array $options) {}

        final public function addKeyAltName(\MongoDB\BSON\Binary $keyId, string $keyAltName): null|object
        {
        }

        final public function createDataKey(string $kmsProvider, null|array $options = null): \MongoDB\BSON\Binary
        {
        }

        final public function decrypt(\MongoDB\BSON\Binary $value): mixed
        {
        }

        final public function deleteKey(\MongoDB\BSON\Binary $keyId): object
        {
        }

        final public function encrypt(mixed $value, null|array $options = null): \MongoDB\BSON\Binary
        {
        }

        final public function encryptExpression(array|object $expr, null|array $options = null): object
        {
        }

        final public function getKey(\MongoDB\BSON\Binary $keyId): null|object
        {
        }

        final public function getKeyByAltName(string $keyAltName): null|object
        {
        }

        final public function getKeys(): Cursor
        {
        }

        final public function removeKeyAltName(\MongoDB\BSON\Binary $keyId, string $keyAltName): null|object
        {
        }

        final public function rewrapManyDataKey(array|object $filter, null|array $options = null): object
        {
        }
    }
}

namespace MongoDB\Driver\Monitoring {
    interface Subscriber
    {
    }

    interface CommandSubscriber extends Subscriber
    {
        public function commandFailed(\MongoDB\Driver\Monitoring\CommandFailedEvent $event): void;

        public function commandStarted(\MongoDB\Driver\Monitoring\CommandStartedEvent $event): void;

        public function commandSucceeded(\MongoDB\Driver\Monitoring\CommandSucceededEvent $event): void;
    }

    interface SDAMSubscriber extends Subscriber
    {
        public function serverChanged(\MongoDB\Driver\Monitoring\ServerChangedEvent $event): void;

        public function serverClosed(\MongoDB\Driver\Monitoring\ServerClosedEvent $event): void;

        public function serverHeartbeatFailed(\MongoDB\Driver\Monitoring\ServerHeartbeatFailedEvent $event): void;

        public function serverHeartbeatStarted(\MongoDB\Driver\Monitoring\ServerHeartbeatStartedEvent $event): void;

        public function serverHeartbeatSucceeded(\MongoDB\Driver\Monitoring\ServerHeartbeatSucceededEvent $event): void;

        public function serverOpening(\MongoDB\Driver\Monitoring\ServerOpeningEvent $event): void;

        public function topologyChanged(\MongoDB\Driver\Monitoring\TopologyChangedEvent $event): void;

        public function topologyClosed(\MongoDB\Driver\Monitoring\TopologyClosedEvent $event): void;

        public function topologyOpening(\MongoDB\Driver\Monitoring\TopologyOpeningEvent $event): void;
    }

    final class CommandStartedEvent
    {
        final public function getCommand(): object
        {
        }

        final public function getCommandName(): string
        {
        }

        final public function getDatabaseName(): string
        {
        }

        final public function getOperationId(): string
        {
        }

        final public function getRequestId(): string
        {
        }

        final public function getServer(): \MongoDB\Driver\Server
        {
        }

        final public function getServiceId(): null|\MongoDB\BSON\ObjectId
        {
        }

        final public function getServerConnectionId(): null|int
        {
        }
    }

    final class CommandSucceededEvent
    {
        final public function getCommandName(): string
        {
        }

        final public function getDurationMicros(): int
        {
        }

        final public function getOperationId(): string
        {
        }

        final public function getReply(): object
        {
        }

        final public function getRequestId(): string
        {
        }

        final public function getServer(): \MongoDB\Driver\Server
        {
        }

        final public function getServiceId(): null|\MongoDB\BSON\ObjectId
        {
        }

        final public function getServerConnectionId(): null|int
        {
        }
    }

    final class CommandFailedEvent
    {
        final public function getCommandName(): string
        {
        }

        final public function getDurationMicros(): int
        {
        }

        final public function getError(): \Throwable
        {
        }

        final public function getOperationId(): string
        {
        }

        final public function getReply(): object
        {
        }

        final public function getRequestId(): string
        {
        }

        final public function getServer(): \MongoDB\Driver\Server
        {
        }

        final public function getServiceId(): null|\MongoDB\BSON\ObjectId
        {
        }

        final public function getServerConnectionId(): null|int
        {
        }
    }

    final class ServerChangedEvent
    {
        final public function getHost(): string
        {
        }

        final public function getNewDescription(): \MongoDB\Driver\ServerDescription
        {
        }

        final public function getPort(): int
        {
        }

        final public function getPreviousDescription(): \MongoDB\Driver\ServerDescription
        {
        }

        final public function getTopologyId(): \MongoDB\BSON\ObjectId
        {
        }
    }

    final class ServerClosedEvent
    {
        final public function getHost(): string
        {
        }

        final public function getPort(): int
        {
        }

        final public function getTopologyId(): \MongoDB\BSON\ObjectId
        {
        }
    }

    final class ServerOpeningEvent
    {
        final public function getHost(): string
        {
        }

        final public function getPort(): int
        {
        }

        final public function getTopologyId(): \MongoDB\BSON\ObjectId
        {
        }
    }

    final class ServerHeartbeatStartedEvent
    {
        final public function getHost(): string
        {
        }

        final public function getPort(): int
        {
        }

        final public function isAwaited(): bool
        {
        }
    }

    final class ServerHeartbeatSucceededEvent
    {
        final public function getDurationMicros(): int
        {
        }

        final public function getHost(): string
        {
        }

        final public function getPort(): int
        {
        }

        final public function getReply(): object
        {
        }

        final public function isAwaited(): bool
        {
        }
    }

    final class ServerHeartbeatFailedEvent
    {
        final public function getDurationMicros(): int
        {
        }

        final public function getError(): \Throwable
        {
        }

        final public function getHost(): string
        {
        }

        final public function getPort(): int
        {
        }

        final public function isAwaited(): bool
        {
        }
    }

    final class TopologyChangedEvent
    {
        final public function getNewDescription(): \MongoDB\Driver\TopologyDescription
        {
        }

        final public function getPreviousDescription(): \MongoDB\Driver\TopologyDescription
        {
        }

        final public function getTopologyId(): \MongoDB\BSON\ObjectId
        {
        }
    }

    final class TopologyClosedEvent
    {
        final public function getTopologyId(): \MongoDB\BSON\ObjectId
        {
        }
    }

    final class TopologyOpeningEvent
    {
        final public function getTopologyId(): \MongoDB\BSON\ObjectId
        {
        }
    }

    function addSubscriber(Subscriber $subscriber): void
    {
    }

    function removeSubscriber(Subscriber $subscriber): void
    {
    }
}
