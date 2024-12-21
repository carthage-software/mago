<?php

namespace Relay;

/**
 * Relay Cluster client.
 *
 * @see https://redis.io/docs/management/scaling/
 */
class Cluster
{
    /**
     * Integer representing the failover option.
     *
     * @var int
     */
    public const OPT_SLAVE_FAILOVER = 5;

    /**
     * Integer representing no failover.
     *
     * Enabled by default. Send commands to master nodes only.
     *
     * @var int
     */
    public const FAILOVER_NONE = 0;

    /**
     * Integer representing error failover.
     *
     * Send readonly commands to slave nodes if master is unreachable.
     *
     * @var int
     */
    public const FAILOVER_ERROR = 1;

    /**
     * Integer representing distribute failover.
     *
     * Always distribute readonly commands between master and slaves, at random
     *
     * @var int
     */
    public const FAILOVER_DISTRIBUTE = 2;

    /**
     * Integer representing distribute slaves failover.
     *
     * Always distribute readonly commands to the slaves, at random
     *
     * @var int
     */
    public const FAILOVER_DISTRIBUTE_SLAVES = 3;

    /**
     * Create a cluster object.
     *
     * @param  string|null  $name
     * @param  array|null  $seeds
     * @param  int|float  $connect_timeout
     * @param  int|float  $command_timeout
     * @param  bool  $persistent
     * @param  mixed  $auth
     * @param  array|null  $context
     */
    #[\Relay\Attributes\Server]
    public function __construct(
        string|null $name,
        array|null $seeds = null,
        int|float $connect_timeout = 0,
        int|float $command_timeout = 0,
        bool $persistent = false,
        #[\SensitiveParameter] mixed $auth = null,
        array|null $context = null
    ) {}

    /**
     * Compress data with Relay's currently configured compression algorithm.
     *
     * @param  string  $value
     * @return string
     */
    #[\Relay\Attributes\Local]
    public function _compress(string $value): string {}

    /**
     * Returns the number of milliseoconds since Relay has received a reply from the cluster.
     *
     * @return int
     */
    #[\Relay\Attributes\Local]
    public function idleTime(): int {}

    /**
     * Returns an array of endpoints along with each of their keys cached in runtime memory.
     *
     * @internal Temporary debug helper. Do not use.
     * @return array|false
     */
    public function _getKeys(): array|false {}

    /**
     * Return a list of master nodes
     *
     * @return array
     */
    #[\Relay\Attributes\Local]
    public function _masters(): array {}

    /**
     * Returns the serialized and compressed value.
     *
     * @param  mixed  $value
     * @return string
     */
    #[\Relay\Attributes\Local]
    public function _pack(mixed $value): string {}

    /**
     * Returns the value with the prefix.
     *
     * @param  mixed  $value
     * @return string
     */
    #[\Relay\Attributes\Local]
    public function _prefix(mixed $value): string {}

    /**
     * Returns the serialized value.
     *
     * @param  mixed  $value
     * @return string
     */
    #[\Relay\Attributes\Local]
    public function _serialize(mixed $value): string {}

    /**
     * Uncompress data with Relay's currently configured compression algorithm.
     *
     * @param  string  $value
     * @return string
     */
    #[\Relay\Attributes\Local]
    public function _uncompress(string $value): string {}

    /**
     * Returns the unserialized and decompressed value.
     *
     * @param  string  $value
     * @return mixed
     */
    #[\Relay\Attributes\Local]
    public function _unpack(string $value): mixed {}

    /**
     * Returns the unserialized value.
     *
     * @param  string  $value
     * @return mixed
     */
    #[\Relay\Attributes\Local]
    public function _unserialize(string $value): mixed {}

    /**
     * Interact with Redis' ACLs
     *
     * @param  string  $operation
     * @param  string  $args,...
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function acl(array|string $key_or_address, string $operation, string ...$args): mixed {}

    /**
     * Adds allow pattern(s). Only matching keys will be cached in memory.
     *
     * @param  string  $pattern,...
     * @return int
     */
    #[\Relay\Attributes\Local]
    public function addAllowPatterns(string ...$pattern): int {}

    /**
     * Adds ignore pattern(s). Matching keys will not be cached in memory.
     *
     * @param  string  $pattern,...
     * @return int
     */
    #[\Relay\Attributes\Local]
    public function addIgnorePatterns(string ...$pattern): int {}

    /**
     * If key already exists and is a string, this command appends
     * the value at the end of the string. If key does not exist
     * it is created and set as an empty string, so APPEND will
     * be similar to SET in this special case.
     *
     * @param  mixed  $key
     * @param  mixed  $value
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function append(mixed $key, mixed $value): Cluster|int|false {}

    /**
     * Asynchronously rewrite the append-only file.
     *
     * @param  array|string  $key_or_address
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function bgrewriteaof(array|string $key_or_address): Cluster|bool {}

    /**
     * Paus the client until sufficient local and/or remote AOF data has been flushed to disk.
     *
     * @param  array|string  $key_or_address
     * @param  int  $numlocal
     * @param  int  $numremote
     * @return Relay|array
     */
    #[\Relay\Attributes\RedisCommand]
    public function waitaof(array|string $key_or_address, int $numlocal, int $numremote, int $timeout): Relay|array|false {}

    /**
     * Asynchronously save the dataset to disk.
     *
     * @param  array|string  $key_or_address
     * @param  bool  $schedule
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function bgsave(array|string $key_or_address, bool $schedule = false): Cluster|bool {}

    /**
     * Count the number of set bits (population counting) in a string.
     *
     * @param  mixed  $key
     * @param  int  $start
     * @param  int  $end
     * @param  bool  $by_bit
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function bitcount(mixed $key, int $start = 0, int $end = -1, bool $by_bit = false): Cluster|int|false {}

    /**
     * Perform a bitwise operation on one or more keys, storing the result in a new key.
     *
     * @param  string  $operation
     * @param  string  $dstkey
     * @param  string  $srckey
     * @param  string  $other_keys,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function bitop(string $operation, string $dstkey, string $srckey, string ...$other_keys): Cluster|int|false {}

    /**
     * Return the position of the first bit set to 1 or 0 in a string.
     *
     * @param  mixed  $key
     * @param  int  $bit
     * @param  int  $start
     * @param  int  $end
     * @param  bool  $by_bit
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function bitpos(mixed $key, int $bit, int $start = null, int $end = null, bool $by_bit = false): Cluster|int|false {}

    /**
     * BLMOVE is the blocking variant of LMOVE. When source contains elements,
     * this command behaves exactly like LMOVE. When used inside a
     * MULTI/EXEC block, this command behaves exactly like LMOVE.
     *
     * @param  mixed  $srckey
     * @param  mixed  $dstkey
     * @param  string  $srcpos
     * @param  string  $dstpos
     * @param  float  $timeout
     * @return Cluster|string|null|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function blmove(mixed $srckey, mixed $dstkey, string $srcpos, string $dstpos, float $timeout): Cluster|string|null|false {}

    /**
     * Pop elements from a list, or block until one is available
     *
     * @param  float  $timeout
     * @param  array  $keys
     * @param  string  $from
     * @param  int  $count
     * @return Cluster|array|null|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function blmpop(float $timeout, array $keys, string $from, int $count = 1): mixed {}

    /**
     * BLPOP is a blocking list pop primitive. It is the blocking version of LPOP because
     * it blocks the connection when there are no elements to pop from any of the given lists.
     *
     * @param  string|array  $key
     * @param  string|float  $timeout_or_key
     * @param  array  $extra_args,...
     * @return Cluster|array|null|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function blpop(string|array $key, string|float $timeout_or_key, mixed ...$extra_args): Cluster|array|null|false {}

    /**
     * BRPOP is a blocking list pop primitive. It is the blocking version of RPOP because
     * it blocks the connection when there are no elements to pop from any of the given lists.
     *
     * @param  string|array  $key
     * @param  string|float  $timeout_or_key
     * @param  array  $extra_args,...
     * @return Cluster|array|null|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function brpop(string|array $key, string|float $timeout_or_key, mixed ...$extra_args): Cluster|array|null|false {}

    /**
     * Atomically returns and removes the last element (tail) of the list stored at source,
     * and pushes the element at the first element (head) of the list stored at destination.
     * This command will block for an element up to the provided timeout.
     *
     * @param  mixed  $srckey
     * @param  mixed  $dstkey
     * @param  float  $timeout
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function brpoplpush(mixed $srckey, mixed $dstkey, float $timeout): mixed {}

    /**
     * Remove and return members with scores in a sorted set or block until one is available
     *
     * @param  float  $timeout
     * @param  array  $keys
     * @param  string  $from
     * @param  int  $count
     * @return Cluster|array|null|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function bzmpop(float $timeout, array $keys, string $from, int $count = 1): Cluster|array|null|false {}

    /**
     * BZPOPMAX is the blocking variant of the sorted set ZPOPMAX primitive.
     *
     * @param  string|array  $key
     * @param  string|float  $timeout_or_key
     * @param  array  $extra_args,...
     * @return Cluster|array|null|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function bzpopmax(string|array $key, string|float $timeout_or_key, mixed ...$extra_args): Cluster|array|null|false {}

    /**
     * BZPOPMIN is the blocking variant of the sorted set ZPOPMIN primitive.
     *
     * @param  string|array  $key
     * @param  string|float  $timeout_or_key
     * @param  array  $extra_args,...
     * @return Cluster|array|null|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function bzpopmin(string|array $key, string|float $timeout_or_key, mixed ...$extra_args): Cluster|array|null|false {}

    /**
     * Clears the last error that is set, if any.
     *
     * @return bool
     */
    #[\Relay\Attributes\Local]
    public function clearLastError(): bool {}

    /**
     * @return bool
     */
    #[\Relay\Attributes\Local]
    public function clearTransferredBytes(): bool {}

    /**
     * Executes `CLIENT` command operations.
     *
     * @param  array|string  $key_or_address
     * @param  string  $operation
     * @param  mixed  $args,...
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function client(array|string $key_or_address, string $operation, mixed ...$args): mixed {}

    /**
     * Closes the current connection, if it's persistent.
     *
     * @return bool
     */
    #[\Relay\Attributes\Local]
    public function close(): bool {}

    /**
     * Executes `CLUSTER` command operations.
     *
     * @param  array|string  $key_or_address
     * @param  string  $operation
     * @param  mixed  $args,...
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function cluster(array|string $key_or_address, string $operation, mixed ...$args): mixed {}

    /**
     * This is a container command for runtime configuration commands.
     *
     * @param  array|string  $key_or_address
     * @param  string  $operation
     * @param  mixed  $args,...
     * @return Cluster|array|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function config(array|string $key_or_address, string $operation, mixed ...$args): mixed {}

    /**
     * Return an array with details about every Redis command.
     *
     * @param  array  $args,...
     * @return Cluster|array|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function command(mixed ...$args): Cluster|array|int|false {}

    /**
     * This command copies the value stored at the source key to the destination key.
     *
     * @param  mixed  $srckey
     * @param  mixed  $dstkey
     * @param  array|null  $options
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function copy(mixed $srckey, mixed $dstkey, array|null $options = null): Cluster|bool {}

    /**
     * Returns the number of keys in the currently-selected database.
     *
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function dbsize(array|string $key_or_address): Cluster|int|false {}

    /**
     * Decrements the number stored at key by one.
     *
     * @param  mixed  $key
     * @param  int  $by
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function decr(mixed $key, int $by = 1): Cluster|int|false {}

    /**
     * Decrements the number stored at key by decrement.
     *
     * @param  mixed  $key
     * @param  int  $value
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function decrby(mixed $key, int $value): Cluster|int|false {}

    /**
     * Removes the specified keys.
     *
     * @param  mixed  $keys,...
     * @return Cluster|int|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function del(mixed ...$keys): Cluster|int|bool {}

    /**
     * Flushes all previously queued commands in a transaction and restores the connection state to normal.
     * If WATCH was used, DISCARD unwatches all keys watched by the connection.
     *
     * @return bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function discard(): bool {}

    /**
     * Dispatches all pending events.
     *
     * @return int|false
     */
    #[\Relay\Attributes\Local]
    public function dispatchEvents(): int|false {}

    /**
     * Serialize and return the value stored at key in a Redis-specific format.
     *
     * @param  mixed  $key
     * @return Cluster|string|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function dump(mixed $key): Cluster|string|false {}

    /**
     * Asks Redis to echo back the provided string.
     *
     * @param  array|string  $key_or_address
     * @param  string  $message
     * @return Cluster|string|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function echo(array|string $key_or_address, string $message): Cluster|string|false {}

    /**
     * Returns the connection's endpoint identifier.
     *
     * @return array|false
     */
    #[\Relay\Attributes\Local]
    public function endpointId(): array|false {}

    /**
     * Evaluate script using the Lua interpreter.
     *
     * @see https://redis.io/commands/eval
     *
     * @param  mixed  $script
     * @param  array  $args
     * @param  int  $num_keys
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function eval(mixed $script, array $args = [], int $num_keys = 0): mixed {}

    /**
     * Evaluate script using the Lua interpreter.  This is just the "read-only" variant of EVAL
     * meaning it can be run on read-only replicas.
     *
     * @see https://redis.io/commands/eval_ro
     *
     * @param  mixed  $script
     * @param  array  $args
     * @param  int  $num_keys
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function eval_ro(mixed $script, array $args = [], int $num_keys = 0): mixed {}

    /**
     * Evaluates a script cached on the server-side by its SHA1 digest.
     *
     * @param  string  $sha
     * @param  array  $args
     * @param  int  $num_keys
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function evalsha(string $sha, array $args = [], int $num_keys = 0): mixed {}

    /**
     * Evaluates a script cached on the server-side by its SHA1 digest.  This is just the "read-only" variant
     * of `EVALSHA` meaning it can be run on read-only replicas.
     *
     * @param  string  $sha
     * @param  array  $args
     * @param  int  $num_keys
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function evalsha_ro(string $sha, array $args = [], int $num_keys = 0): mixed {}

    /**
     * Executes all previously queued commands in a transaction and restores the connection state to normal.
     *
     * @return array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function exec(): array|false {}

    /**
     * Returns if key(s) exists.
     *
     * @param  mixed  $keys,...
     * @return Cluster|int|bool
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function exists(mixed ...$keys): Cluster|int|bool {}

    /**
     * Set a timeout on key.
     *
     * @param  mixed  $key
     * @param  int  $seconds
     * @param  string|null  $mode
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function expire(mixed $key, int $seconds, string|null $mode = null): Cluster|bool {}

    /**
     * Set a timeout on key using a unix timestamp.
     *
     * @param  mixed  $key
     * @param  int  $timestamp
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function expireat(mixed $key, int $timestamp): Cluster|bool {}

    /**
     * Returns the absolute Unix timestamp in seconds at which the given key will expire.
     * If the key exists but doesn't have a TTL this function return -1.
     * If the key does not exist -2.
     *
     * @param  mixed  $key
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function expiretime(mixed $key): Cluster|int|false {}

    /**
     * @see \Relay\Relay::flushMemory()
     *
     * @param  string|null  $endpointId
     * @param  int|null  $db
     * @return bool
     */
    #[\Relay\Attributes\Local]
    public static function flushMemory(?string $endpointId = null, int $db = null): bool {}

    /**
     * Deletes all the keys of all the existing databases, not just the currently selected one.
     *
     * @param  array|string  $key_or_address
     * @param  bool|null  $sync
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function flushall(array|string $key_or_address, bool|null $sync = null): Cluster|bool {}

    /**
     * Deletes all the keys of the currently selected database.
     *
     * @param  array|string  $key_or_address
     * @param  bool|null  $sync
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function flushdb(array|string $key_or_address, bool|null $sync = null): Cluster|bool {}

    /**
     * Add one or more members to a geospacial sorted set
     *
     * @param  mixed  $key
     * @param  float  $lng
     * @param  float  $lat
     * @param  string  $member
     * @param  mixed  $other_triples_and_options,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function geoadd(mixed $key, float $lng, float $lat, string $member, mixed ...$other_triples_and_options): Cluster|int|false {}

    /**
     * Get the distance between two members of a geospacially encoded sorted set.
     *
     * @param  mixed  $key
     * @param  string  $src
     * @param  string  $dst
     * @param  string|null  $unit
     * @return Cluster|float|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function geodist(mixed $key, string $src, string $dst, string|null $unit = null): Cluster|float|false {}

    /**
     * Retrieve one or more GeoHash encoded strings for members of the set.
     *
     * @param  mixed  $key
     * @param  string  $member
     * @param  string  $other_members,...
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function geohash(mixed $key, string $member, string ...$other_members): Cluster|array|false {}

    /**
     * Return the positions (longitude,latitude) of all the specified members
     * of the geospatial index represented by the sorted set at key.
     *
     * @param  mixed  $key
     * @param  mixed  $members,...
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function geopos(mixed $key, mixed ...$members): Cluster|array|false {}

    /**
     * Retrieve members of a geospacially sorted set that are within a certain radius of a location.
     *
     * @param  mixed  $key
     * @param  float  $lng
     * @param  float  $lat
     * @param  float  $radius
     * @param  string  $unit
     * @param  array  $options
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function georadius(mixed $key, float $lng, float $lat, float $radius, string $unit, array $options = []): mixed {}

    /**
     * Retrieve members of a geospacially sorted set that are within a certain radius of a location.
     *
     * @param  mixed  $key
     * @param  float  $lng
     * @param  float  $lat
     * @param  float  $radius
     * @param  string  $unit
     * @param  array  $options
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function georadius_ro(mixed $key, float $lng, float $lat, float $radius, string $unit, array $options = []): mixed {}

    /**
     * Similar to `GEORADIUS` except it uses a member as the center of the query.
     *
     * @param  mixed  $key
     * @param  string  $member
     * @param  float  $radius
     * @param  string  $unit
     * @param  array  $options
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function georadiusbymember(mixed $key, string $member, float $radius, string $unit, array $options = []): mixed {}

    /**
     * Similar to `GEORADIUS` except it uses a member as the center of the query.
     *
     * @param  mixed  $key
     * @param  string  $member
     * @param  float  $radius
     * @param  string  $unit
     * @param  array  $options
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function georadiusbymember_ro(mixed $key, string $member, float $radius, string $unit, array $options = []): mixed {}

    /**
     * Search a geospacial sorted set for members in various ways.
     *
     * @param  mixed  $key
     * @param  array|string  $position
     * @param  array|int|float  $shape
     * @param  string  $unit
     * @param  array  $options
     * @return Cluster|array
     */
    #[\Relay\Attributes\RedisCommand]
    public function geosearch(mixed $key, array|string $position, array|int|float $shape, string $unit, array $options = []): Cluster|array {}

    /**
     * Search a geospacial sorted set for members within a given area or range, storing the results into
     * a new set.
     *
     * @param  mixed  $dstkey
     * @param  mixed  $srckey
     * @param  array|string  $position
     * @param  array|int|float  $shape
     * @param  string  $unit
     * @param  array  $options
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function geosearchstore(mixed $dstkey, mixed $srckey, array|string $position, array|int|float $shape, string $unit, array $options = []): Cluster|int|false {}

    /**
     * Get the value of key.
     *
     * @param  mixed  $key
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function get(mixed $key): mixed {}

    /**
     * Returns the bit value at offset in the string value stored at key.
     *
     * @param  mixed  $key
     * @param  int  $pos
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function getbit(mixed $key, int $pos): Cluster|int|false {}

    /**
     * Get the value of key and optionally set its expiration.
     * GETEX is similar to GET, but is a write command with additional options.
     *
     * @param  mixed  $key
     * @param  array  $options
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function getex(mixed $key, ?array $options = null): mixed {}

    /**
     * Returns the last error message, if any.
     *
     * @return string|null
     */
    #[\Relay\Attributes\Local]
    public function getLastError(): string|null {}

    /**
     * Get the mode Relay is currently in.
     * `Relay::ATOMIC` or `Relay::MULTI`.
     *
     * @param  bool  $masked
     * @return int
     */
    #[\Relay\Attributes\Local]
    public function getMode(bool $masked = false): int {}

    /**
     * Returns a client option.
     *
     * @param  int  $option
     * @return mixed
     */
    #[\Relay\Attributes\Local]
    public function getOption(int $option): mixed {}

    /**
     * @return array|false
     */
    #[\Relay\Attributes\Local]
    public function getTransferredBytes(): array|false {}

    /**
     * Returns the substring of the string value stored at key,
     * determined by the offsets start and end (both are inclusive).
     *
     * @param  mixed  $key
     * @param  int  $start
     * @param  int  $end
     * @return Cluster|string|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function getrange(mixed $key, int $start, int $end): Cluster|string|false {}

    /**
     * Atomically sets key to value and returns the old value stored at key.
     *
     * @param  mixed  $key
     * @param  mixed  $value
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function getset(mixed $key, mixed $value): mixed {}

    /**
     * Removes the specified fields from the hash stored at key.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @param  string  $members,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function hdel(mixed $key, mixed $member, mixed ...$members): Cluster|int|false {}

    /**
     * Returns if field is an existing field in the hash stored at key.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function hexists(mixed $key, mixed $member): Cluster|bool {}

    /**
     * Returns the value associated with field in the hash stored at key.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function hget(mixed $key, mixed $member): mixed {}

    /**
     * Returns all fields and values of the hash stored at key.
     *
     * @param  mixed  $key
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function hgetall(mixed $key): Cluster|array|false {}

    /**
     * Increments the number stored at field in the hash stored at key by increment.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @param  int  $value
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function hincrby(mixed $key, mixed $member, int $value): Cluster|int|false {}

    /**
     * Increment the specified field of a hash stored at key, and representing
     * a floating point number, by the specified increment.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @param  float  $value
     * @return Cluster|float|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function hincrbyfloat(mixed $key, mixed $member, float $value): Cluster|float|bool {}

    /**
     * Returns all field names in the hash stored at key.
     *
     * @param  mixed  $key
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function hkeys(mixed $key): Cluster|array|false {}

    /**
     * Returns the number of fields contained in the hash stored at `$key`.
     *
     * @param  mixed  $key
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function hlen(mixed $key): Cluster|int|false {}

    /**
     * Returns the values associated with the specified fields in the hash stored at key.
     *
     * @param  mixed  $key
     * @param  array  $members
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function hmget(mixed $key, array $members): Cluster|array|false {}

    /**
     * Sets the specified fields to their respective values in the hash stored at key.
     *
     * @param  mixed  $key
     * @param  array  $members
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function hmset(mixed $key, array $members): Cluster|bool {}

    /**
     * When called with just the key argument, return a random field from the hash value stored at key.
     *
     * @param  mixed  $key
     * @param  array  $options
     * @return Cluster|array|string|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function hrandfield(mixed $key, array|null $options = null): Cluster|array|string|false {}

    /**
     * Iterates fields of Hash types and their associated values.
     *
     * @param  mixed  $key
     * @param  mixed  $iterator
     * @param  mixed  $match
     * @param  int  $count
     * @return array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function hscan(mixed $key, mixed &$iterator, mixed $match = null, int $count = 0): array|false {}

    /**
     * Sets field in the hash stored at key to value.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @param  mixed  $value
     * @param  mixed  $kvals,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function hset(mixed $key, mixed $member, mixed $value, mixed ...$kvals): Cluster|int|false {}

    /**
     * Sets field in the hash stored at key to value, only if field does not yet exist.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @param  mixed  $value
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function hsetnx(mixed $key, mixed $member, mixed $value): Cluster|bool {}

    /**
     * Returns the string length of the value associated with field in the hash stored at key.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function hstrlen(mixed $key, mixed $member): Cluster|int|false {}

    /**
     * Returns all values in the hash stored at key.
     *
     * @param  mixed  $key
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function hvals(mixed $key): Cluster|array|false {}

    /**
     * Increments the number stored at key by one.
     *
     * @param  mixed  $key
     * @param  int  $by
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function incr(mixed $key, int $by = 1): Cluster|int|false {}

    /**
     * Increments the number stored at key by increment.
     *
     * @param  mixed  $key
     * @param  int  $value
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function incrby(mixed $key, int $value): Cluster|int|false {}

    /**
     * Increment the string representing a floating point number stored at key by the specified increment.
     *
     * @param  mixed  $key
     * @param  float  $value
     * @return Cluster|float|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function incrbyfloat(mixed $key, float $value): Cluster|float|false {}

    /**
     * The INFO command returns information and statistics about Redis in a format
     * that is simple to parse by computers and easy to read by humans.
     *
     * @see https://redis.io/commands/info
     *
     * @param  array|string  $key_or_address
     * @param  string  $sections,...
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function info(array|string $key_or_address, string ...$sections): Cluster|array|false {}

    /**
     * Returns all keys matching pattern.
     *
     * @param  mixed  $pattern
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function keys(mixed $pattern): Cluster|array|false {}

    /**
     * Returns the UNIX time stamp of the last successful save to disk.
     *
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function lastsave(array|string $key_or_address): Cluster|int|false {}

    /**
     * Get the longest common subsequence between two string keys.
     *
     * @param  mixed  $key1
     * @param  mixed  $key2
     * @param  array|null  $options
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function lcs(mixed $key1, mixed $key2, array|null $options = null): mixed {}

    /**
     * Returns the element at index index in the list stored at key.
     *
     * @param  mixed  $key
     * @param  int  $index
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function lindex(mixed $key, int $index): mixed {}

    /**
     * Inserts element in the list stored at key either before or after the reference value pivot.
     *
     * @param  mixed  $key
     * @param  string  $op
     * @param  mixed  $pivot
     * @param  mixed  $element
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function linsert(mixed $key, string $op, mixed $pivot, mixed $element): Cluster|int|false {}

    /**
     * Registers a new event listener.
     *
     * @param  callable  $callback
     * @return bool
     */
    #[\Relay\Attributes\Local]
    public function listen(?callable $callback): bool {}

    /**
     * Returns the length of the list stored at `$key`.
     *
     * @param  mixed  $key
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function llen(mixed $key): Cluster|int|false {}

    /**
     * Atomically returns and removes the first/last element of the list
     * stored at source, and pushes the element at the first/last
     * element of the list stored at destination.
     *
     * @param  mixed  $srckey
     * @param  mixed  $dstkey
     * @param  string  $srcpos
     * @param  string  $dstpos
     * @return Cluster|string|null|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function lmove(mixed $srckey, mixed $dstkey, string $srcpos, string $dstpos): Cluster|string|null|false {}

    /**
     * Pops one or more elements from the first non-empty list key from the list of provided key names.
     *
     * @param  array  $keys
     * @param  string  $from
     * @param  int  $count
     * @return Cluster|array|null|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function lmpop(array $keys, string $from, int $count = 1): mixed {}

    /**
     * Removes and returns the first elements of the list stored at key.
     *
     * @param  mixed  $key
     * @param  int  $count
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function lpop(mixed $key, int $count = 1): mixed {}

    /**
     * The command returns the index of matching elements inside a Redis list.
     *
     * @param  mixed  $key
     * @param  mixed  $value
     * @param  array  $options
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function lpos(mixed $key, mixed $value, array|null $options = null): mixed {}

    /**
     * Insert all the specified values at the head of the list stored at key.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @param  mixed  $members,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function lpush(mixed $key, mixed $member, mixed ...$members): Cluster|int|false {}

    /**
     * Inserts specified values at the head of the list stored at key,
     * only if key already exists and holds a list.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @param  mixed  $members,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function lpushx(mixed $key, mixed $member, mixed ...$members): Cluster|int|false {}

    /**
     * Returns the specified elements of the list stored at key.
     *
     * @param  mixed  $key
     * @param  int  $start
     * @param  int  $stop
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function lrange(mixed $key, int $start, int $stop): Cluster|array|false {}

    /**
     * Removes the first count occurrences of elements equal to element from the list stored at key.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @param  int  $count
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function lrem(mixed $key, mixed $member, int $count = 0): Cluster|int|false {}

    /**
     * Sets the list element at index to element.
     *
     * @param  mixed  $key
     * @param  int  $index
     * @param  mixed  $member
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function lset(mixed $key, int $index, mixed $member): Cluster|bool {}

    /**
     * Trim an existing list so that it will contain only the specified range of elements specified.
     *
     * @param  mixed  $key
     * @param  int  $start
     * @param  int  $end
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function ltrim(mixed $key, int $start, int $end): Cluster|bool {}

    /**
     * Returns the number of bytes allocated, or `0` in client-only mode.
     *
     * @return int
     */
    #[\Relay\Attributes\Local]
    public static function maxMemory(): int {}

    /**
     * Returns the values of all specified keys.
     *
     * @param  array  $keys
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function mget(array $keys): Cluster|array|false {}

    /**
     * Sets the given keys to their respective values.
     * MSET replaces existing values with new values, just as regular SET.
     *
     * @param  array  $kvals
     * @return Cluster|array|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function mset(array $kvals): Cluster|array|bool {}

    /**
     * Sets the given keys to their respective values.
     * MSETNX will not perform any operation at all even if just a single key already exists.
     *
     * @param  array  $kvals
     * @return Cluster|array|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function msetnx(array $kvals): Cluster|array|bool {}

    /**
     * Marks the start of a transaction block. Subsequent commands will be queued for atomic execution using EXEC.
     *
     * Accepts only `Relay::MULTI` mode.
     *
     * @param  int  $mode
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function multi(int $mode = Relay::MULTI): Cluster|bool {}

    /**
     * This is a container command for object introspection commands.
     *
     * @param  string  $op
     * @param  mixed  $key
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function object(string $op, mixed $key): mixed {}

    /**
     * Registers a new `flushed` event listener.
     *
     * @param  callable  $callback
     * @return bool
     */
    #[\Relay\Attributes\Local]
    public function onFlushed(?callable $callback): bool {}

    /**
     * Registers a new `invalidated` event listener.
     *
     * @param  callable  $callback
     * @param  string|null  $pattern
     * @return bool
     */
    #[\Relay\Attributes\Local]
    public function onInvalidated(?callable $callback, ?string $pattern = null): bool {}

    /**
     * Remove the existing timeout on key, turning the key from volatile to persistent.
     *
     * @param  mixed  $key
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function persist(mixed $key): Cluster|bool {}

    /**
     * Set a key's time to live in milliseconds.
     *
     * @param  mixed  $key
     * @param  int  $milliseconds
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function pexpire(mixed $key, int $milliseconds): Cluster|bool {}

    /**
     * Set the expiration for a key as a UNIX timestamp specified in milliseconds.
     *
     * @param  mixed  $key
     * @param  int  $timestamp_ms
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function pexpireat(mixed $key, int $timestamp_ms): Cluster|bool {}

    /**
     * Semantic the same as EXPIRETIME, but returns the absolute Unix expiration
     * timestamp in milliseconds instead of seconds.
     *
     * @param  mixed  $key
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function pexpiretime(mixed $key): Cluster|int|false {}

    /**
     * Adds the specified elements to the specified HyperLogLog.
     *
     * @param  string  $key
     * @param  array  $elements
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function pfadd(mixed $key, array $elements): Cluster|int|false {}

    /**
     * Return the approximated cardinality of the set(s) observed by the HyperLogLog at key(s).
     *
     * @param  string  $key
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function pfcount(mixed $key): Cluster|int|false {}

    /**
     * Merge given HyperLogLogs into a single one.
     *
     * @param  string  $dstkey
     * @param  array  $srckeys
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function pfmerge(string $dstkey, array $srckeys): Cluster|bool {}

    /**
     * Returns PONG if no argument is provided, otherwise return a copy of the argument as a bulk.
     *
     * @param  array|string  $key_or_address
     * @param  string|null  $message
     * @return Cluster|bool|string
     */
    #[\Relay\Attributes\RedisCommand]
    public function ping(array|string $key_or_address, string|null $message = null): Cluster|bool|string {}

    /**
     * Set key to hold the string value and set key to timeout after a given number of milliseconds.
     *
     * @param  mixed  $key
     * @param  int  $milliseconds
     * @param  mixed  $value
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function psetex(mixed $key, int $milliseconds, mixed $value): Cluster|bool {}

    /**
     * Subscribes to the given patterns.
     *
     * @param  array  $patterns
     * @param  callable  $callback
     * @return bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function psubscribe(array $patterns, callable $callback): bool {}

    /**
     * Returns the remaining time to live of a key that has a timeout in milliseconds.
     *
     * @param  mixed  $key
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function pttl(mixed $key): Cluster|int|false {}

    /**
     * Posts a message to the given channel.
     *
     * @param  string  $channel
     * @param  string  $message
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function publish(string $channel, string $message): Cluster|int|false {}

    /**
     * A container command for Pub/Sub introspection commands.
     *
     * @param  array|string  $key_or_address
     * @param  string  $operation
     * @param  mixed  $args,...
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function pubsub(array|string $key_or_address, string $operation, mixed ...$args): mixed {}

    /**
     * Unsubscribes from the given patterns, or from all of them if none is given.
     *
     * @param  array  $patterns
     * @return bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function punsubscribe(array $patterns = []): bool {}

    /**
     * Returns a random key from Redis.
     *
     * @param  array|string  $key_or_address
     * @return Cluster|bool|string
     */
    #[\Relay\Attributes\RedisCommand]
    public function randomkey(array|string $key_or_address): Cluster|bool|string {}

    /**
     * Execute any command against Redis, without applying
     * the prefix, compression and serialization.
     *
     * @param  array|string  $key_or_address
     * @param  string  $cmd
     * @param  mixed  $args,...
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function rawCommand(array|string $key_or_address, string $cmd, mixed ...$args): mixed {}

    /**
     * Renames key.
     *
     * @param  mixed  $key
     * @param  mixed  $newkey
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function rename(mixed $key, mixed $newkey): Cluster|bool {}

    /**
     * Renames key if the new key does not yet exist.
     *
     * @param  mixed  $key
     * @param  mixed  $newkey
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function renamenx(mixed $key, mixed $newkey): Cluster|bool {}

    /**
     * Create a key associated with a value that is obtained by deserializing the provided serialized value.
     *
     * @param  mixed  $key
     * @param  int  $ttl
     * @param  string  $value
     * @param  array|null  $options
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function restore(mixed $key, int $ttl, string $value, array|null $options = null): Cluster|bool {}

    /**
     * Returns the role of the instance in the context of replication.
     *
     * @param  array|string  $key_or_address
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function role(array|string $key_or_address): Cluster|array|false {}

    /**
     * Removes and returns the last elements of the list stored at key.
     *
     * @param  mixed  $key
     * @param  int  $count
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function rpop(mixed $key, int $count = 1): mixed {}

    /**
     * Atomically returns and removes the last element (tail) of the list stored at source,
     * and pushes the element at the first element (head) of the list stored at destination.
     *
     * @param  mixed  $srckey
     * @param  mixed  $dstkey
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function rpoplpush(mixed $srckey, mixed $dstkey): mixed {}

    /**
     * Insert all the specified values at the tail of the list stored at key.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @param  mixed  $members,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function rpush(mixed $key, mixed $member, mixed ...$members): Cluster|int|false {}

    /**
     * Inserts specified values at the tail of the list stored at key,
     * only if key already exists and holds a list.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @param  mixed  $members,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function rpushx(mixed $key, mixed $member, mixed ...$members): Cluster|int|false {}

    /**
     * Add the specified members to the set stored at `$key`.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @param  mixed  $members,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function sadd(mixed $key, mixed $member, mixed ...$members): Cluster|int|false {}

    /**
     * Synchronously save the dataset to disk.
     *
     * @param  array|string  $key_or_address
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function save(array|string $key_or_address): Cluster|bool {}

    /**
     * Scan the keyspace for matching keys.
     *
     * @param  mixed  $iterator
     * @param  array|string  $key_or_address
     * @param  mixed  $match
     * @param  int  $count
     * @param  string|null  $type
     * @return array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function scan(mixed &$iterator, array|string $key_or_address, mixed $match = null, int $count = 0, string|null $type = null): array|false {}

    /**
     * Returns the set cardinality (number of elements) of the set stored at `$key`.
     *
     * @param  mixed  $key
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function scard(mixed $key): Cluster|int|false {}

    /**
     * Execute a script management command.
     *
     * @param  array|string  $key_or_address
     * @param  string  $operation
     * @param  string  $args,...
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function script(array|string $key_or_address, string $operation, string ...$args): mixed {}

    /**
     * Returns the members of the set resulting from the difference between the first set and all the successive sets.
     *
     * @param  mixed  $key
     * @param  mixed  $other_keys,...
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function sdiff(mixed $key, mixed ...$other_keys): Cluster|array|false {}

    /**
     * This command is equal to SDIFF, but instead of returning the resulting set, it is stored in destination.
     * If destination already exists, it is overwritten.
     *
     * @param  mixed  $key
     * @param  mixed  $other_keys,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function sdiffstore(mixed $key, mixed ...$other_keys): Cluster|int|false {}

    /**
     * Set key to hold the string value. If key already holds
     * a value, it is overwritten, regardless of its type.
     *
     * @param  mixed  $key
     * @param  mixed  $value
     * @param  mixed  $options
     * @return Cluster|string|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function set(mixed $key, mixed $value, mixed $options = null): Cluster|string|bool {}

    /**
     * Sets a client option.
     *
     * Relay specific options:
     *
     * - `OPT_ALLOW_PATTERNS`
     * - `OPT_IGNORE_PATTERNS`
     * - `OPT_THROW_ON_ERROR`
     * - `OPT_CLIENT_INVALIDATIONS`
     * - `OPT_PHPREDIS_COMPATIBILITY`
     *
     * Supported PhpRedis options:
     *
     * - `OPT_PREFIX`
     * - `OPT_READ_TIMEOUT`
     * - `OPT_COMPRESSION`
     * - `OPT_COMPRESSION_LEVEL`
     * - `OPT_MAX_RETRIES`
     * - `OPT_BACKOFF_ALGORITHM`
     * - `OPT_BACKOFF_BASE`
     * - `OPT_BACKOFF_CAP`
     * - `OPT_SCAN`
     * - `OPT_REPLY_LITERAL`
     * - `OPT_NULL_MULTIBULK_AS_NULL`
     * - `OPT_SLAVE_FAILOVER`
     *
     * @param  int  $option
     * @param  mixed  $value
     * @return bool
     */
    #[\Relay\Attributes\Local]
    public function setOption(int $option, mixed $value): bool {}

    /**
     * Sets or clears the bit at offset in the string value stored at key.
     *
     * @param  mixed  $key
     * @param  int  $pos
     * @param  int  $value
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function setbit(mixed $key, int $pos, int $value): Cluster|int|false {}

    /**
     * Set key to hold the string value and set key to timeout after a given number of seconds.
     *
     * @param  mixed  $key
     * @param  int  $seconds
     * @param  mixed  $value
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function setex(mixed $key, int $seconds, mixed $value): Cluster|bool {}

    /**
     * Set key to hold string value if key does not exist. In that case, it is equal to SET.
     * When key already holds a value, no operation is performed.
     * SETNX is short for "SET if Not eXists".
     *
     * @param  mixed  $key
     * @param  mixed  $value
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function setnx(mixed $key, mixed $value): Cluster|bool {}

    /**
     * Overwrites part of the string stored at key, starting at
     * the specified offset, for the entire length of value.
     *
     * @param  mixed  $key
     * @param  int  $start
     * @param  mixed  $value
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function setrange(mixed $key, int $start, mixed $value): Cluster|int|false {}

    /**
     * Returns the members of the set resulting from the intersection of all the given sets.
     *
     * @param  mixed  $key
     * @param  mixed  $other_keys,...
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function sinter(mixed $key, mixed ...$other_keys): Cluster|array|false {}

    /**
     * Intersect multiple sets and return the cardinality of the result.
     *
     * @param  array  $keys
     * @param  int  $limit
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function sintercard(array $keys, int $limit = -1): Cluster|int|false {}

    /**
     * This command is equal to SINTER, but instead of returning the resulting set, it is stored in destination.
     * If destination already exists, it is overwritten.
     *
     * @param  mixed  $key
     * @param  mixed  $other_keys,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function sinterstore(mixed $key, mixed ...$other_keys): Cluster|int|false {}

    /**
     * Returns if `$member` is a member of the set stored at `$key`.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function sismember(mixed $key, mixed $member): Cluster|bool {}

    /**
     * Interact with the Redis slowlog.
     *
     * @param  array|string  $key_or_address
     * @param  string  $operation
     * @param  mixed  $args,...
     * @return Cluster|array|int|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function slowlog(array|string $key_or_address, string $operation, mixed ...$args): Cluster|array|int|bool {}

    /**
     * Returns all the members of the set value stored at `$key`.
     *
     * @param  mixed  $key
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function smembers(mixed $key): Cluster|array|false {}

    /**
     * Returns whether each member is a member of the set stored at `$key`.
     *
     * @param  mixed  $key
     * @param  mixed  $members,...
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function smismember(mixed $key, mixed ...$members): Cluster|array|false {}

    /**
     * Move member from the set at source to the set at destination.
     *
     * @param  mixed  $srckey
     * @param  mixed  $dstkey
     * @param  mixed  $member
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function smove(mixed $srckey, mixed $dstkey, mixed $member): Cluster|bool {}

    /**
     * Sort the elements in a list, set or sorted set.
     *
     * @param  mixed  $key
     * @param  array  $options
     * @return Cluster|array|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function sort(mixed $key, array $options = []): Cluster|array|int|false {}

    /**
     * Sort the elements in a list, set or sorted set. Read-only variant of SORT.
     *
     * @param  mixed  $key
     * @param  array  $options
     * @return Cluster|array|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function sort_ro(mixed $key, array $options = []): Cluster|array|int|false {}

    /**
     * Removes and returns one or more random members from the set value store at `$key`.
     *
     * @param  mixed  $key
     * @param  int  $count
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function spop(mixed $key, int $count = 1): mixed {}

    /**
     * Returns one or multiple random members from a set.
     *
     * @param  mixed  $key
     * @param  int  $count
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function srandmember(mixed $key, int $count = 1): mixed {}

    /**
     * Remove the specified members from the set stored at `$key`.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @param  mixed  $members,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function srem(mixed $key, mixed $member, mixed ...$members): Cluster|int|false {}

    /**
     * Iterates elements of Sets types.
     *
     * @param  mixed  $key
     * @param  mixed  $iterator
     * @param  mixed  $match
     * @param  int  $count
     * @return array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function sscan(mixed $key, mixed &$iterator, mixed $match = null, int $count = 0): array|false {}

    /**
     * Subscribes to the specified shard channels.
     *
     * @param  array  $channels
     * @param  callable  $callback
     * @return bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function ssubscribe(array $channels, callable $callback): bool {}

    /**
     * Returns statistics about Relay.
     *
     * @see \Relay\Relay::stats()
     * @return array
     */
    #[\Relay\Attributes\Local]
    public static function stats(): array {}

    /**
     * Returns the length of the string value stored at `$key`.
     *
     * @param  mixed  $key
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function strlen(mixed $key): Cluster|int|false {}

    /**
     * Subscribes to the specified channels.
     *
     * @param  array  $channels
     * @param  callable  $callback
     * @return bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function subscribe(array $channels, callable $callback): bool {}

    /**
     * Returns the members of the set resulting from the union of all the given sets.
     *
     * @param  mixed  $key
     * @param  mixed  $other_keys,...
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand, \Relay\Attributes\Cached]
    public function sunion(mixed $key, mixed ...$other_keys): Cluster|array|false {}

    /**
     * This command is equal to SUNION, but instead of returning the resulting set, it is stored in destination.
     * If destination already exists, it is overwritten.
     *
     * @param  mixed  $key
     * @param  mixed  $other_keys,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function sunionstore(mixed $key, mixed ...$other_keys): Cluster|int|false {}

    /**
     * Unsubscribes from the given shard channels, or from all of them if none is given.
     *
     * @param  array  $channels
     * @return bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function sunsubscribe(array $channels = []): bool {}

    /**
     * Returns the current time from Redis.
     *
     * @param  array|string  $key_or_address
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function time(array|string $key_or_address): Cluster|array|false {}

    /**
     * Alters the last access time of a key(s).
     *
     * @param  array|string  $key_or_array
     * @param  mixed  $more_keys,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function touch(array|string $key_or_array, mixed ...$more_keys): Cluster|int|false {}

    /**
     * Returns the remaining time to live of a key that has a timeout in seconds.
     *
     * @param  mixed  $key
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function ttl(mixed $key): Cluster|int|false {}

    /**
     * Returns the type of a given key.
     *
     * In PhpRedis compatibility mode this will return an integer
     * (one of the REDIS_<type>) constants. Otherwise it will
     * return the string that Redis returns.
     *
     * @param  mixed  $key
     * @return Cluster|int|string|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function type(mixed $key): Cluster|int|string|bool {}

    /**
     * Removes the specified keys without blocking Redis.
     *
     * @param  mixed  $keys,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function unlink(mixed ...$keys): Cluster|int|false {}

    /**
     * Unsubscribes from the given channels, or from all of them if none is given.
     *
     * @param  array  $channels
     * @return bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function unsubscribe(array $channels = []): bool {}

    /**
     * Flushes all the previously watched keys for a transaction.
     * If you call EXEC or DISCARD, there's no need to manually call UNWATCH.
     *
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function unwatch(): Cluster|bool {}

    /**
     * Marks the given keys to be watched for conditional execution of a transaction.
     *
     * @param  mixed  $key
     * @param  mixed  $other_keys,...
     * @return Cluster|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function watch(mixed $key, mixed ...$other_keys): Cluster|bool {}

    /**
     * Acknowledge one or more IDs as having been processed by the consumer group.
     *
     * @param  mixed  $key
     * @param  string  $group
     * @param  array  $ids
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function xack(mixed $key, string $group, array $ids): Cluster|int|false {}

    /**
     * Append a message to a stream.
     *
     * @param  string  $key
     * @param  string  $id
     * @param  int  $maxlen
     * @param  bool  $approx
     * @param  bool  $nomkstream
     * @return Cluster|string|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function xadd(mixed $key, string $id, array $values, int $maxlen = 0, bool $approx = false, bool $nomkstream = false): Cluster|string|false {}

    /**
     * Automatically take ownership of stream message(s) by metrics
     *
     * @param  string  $key
     * @param  string  $group
     * @param  string  $consumer
     * @param  int  $min_idle
     * @param  string  $start
     * @param  int  $count
     * @param  bool  $justid
     * @return Cluster|array|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function xautoclaim(mixed $key, string $group, string $consumer, int $min_idle, string $start, int $count = -1, bool $justid = false): Cluster|bool|array {}

    /**
     * Claim ownership of stream message(s).
     *
     * @param  string  $key
     * @param  string  $group
     * @param  string  $consumer
     * @param  int  $min_idle
     * @param  array  $ids
     * @param  array  $options
     * @return Cluster|array|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function xclaim(mixed $key, string $group, string $consumer, int $min_idle, array $ids, array $options): Cluster|array|bool {}

    /**
     * Remove one or more specific IDs from a stream.
     *
     * @param  string  $key
     * @param  array  $ids
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function xdel(mixed $key, array $ids): Cluster|int|false {}

    /**
     * Perform utility operations having to do with consumer groups
     *
     * @param  string  $operation
     * @param  mixed  $key
     * @param  string  $group
     * @param  string  $id_or_consumer
     * @param  bool  $mkstream
     * @param  int  $entries_read
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function xgroup(string $operation, mixed $key = null, string $group = null, string $id_or_consumer = null, bool $mkstream = false, int $entries_read = -2): mixed {}

    /**
     * Retrieve information about a stream key.
     *
     * @param  string  $operation
     * @param  string|null  $arg1
     * @param  string|null  $arg2
     * @param  int  $count
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function xinfo(string $operation, string|null $arg1 = null, string|null $arg2 = null, int $count = -1): mixed {}

    /**
     * Get the length of a stream.
     *
     * @param  mixed  $key
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function xlen(mixed $key): Cluster|int|false {}

    /**
     * Query pending entries in a stream.
     *
     * @param  string  $key
     * @param  string  $group
     * @param  string|null  $start
     * @param  string|null  $end
     * @param  int  $count
     * @param  string|null  $consumer
     * @param  int  $idle
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function xpending(mixed $key, string $group, string|null $start = null, string|null $end = null, int $count = -1, string|null $consumer = null, int $idle = 0): Cluster|array|false {}

    /**
     * Lists elements in a stream.
     *
     * @param  mixed  $key
     * @param  string  $start
     * @param  string  $end
     * @param  int  $count = -1
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function xrange(mixed $key, string $start, string $end, int $count = -1): Cluster|array|false {}

    /**
     * Read messages from a stream.
     *
     * @param  array  $streams
     * @param  int  $count
     * @param  int  $block
     * @return Cluster|array|bool|null
     */
    #[\Relay\Attributes\RedisCommand]
    public function xread(array $streams, int $count = -1, int $block = -1): Cluster|array|bool|null {}

    /**
     * Read messages from a stream using a consumer group.
     *
     * @param  mixed  $key
     * @param  string  $consumer
     * @param  array  $streams
     * @param  int  $count
     * @param  int  $block
     * @return Cluster|array|bool|null
     */
    #[\Relay\Attributes\RedisCommand]
    public function xreadgroup(mixed $key, string $consumer, array $streams, int $count = 1, int $block = 1): Cluster|array|bool|null {}

    /**
     * Get a range of entries from a STREAM ke in reverse chronological order.
     *
     * @param  mixed  $key
     * @param  string  $end
     * @param  string  $start
     * @param  int  $count
     * @return Cluster|array|bool
     */
    #[\Relay\Attributes\RedisCommand]
    public function xrevrange(mixed $key, string $end, string $start, int $count = -1): Cluster|array|bool {}

    /**
     * Truncate a STREAM key in various ways.
     *
     * @param  mixed  $key
     * @param  string  $threshold
     * @param  bool  $approx
     * @param  bool  $minid
     * @param  int  $limit
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function xtrim(mixed $key, string $threshold, bool $approx = false, bool $minid = false, int $limit = -1): Cluster|int|false {}

    /**
     * Adds all the specified members with the specified scores to the sorted set stored at key.
     *
     * @param  mixed  $key
     * @param  mixed  $args,...
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function zadd(mixed $key, mixed ...$args): mixed {}

    /**
     * Returns the sorted set cardinality (number of elements) of the sorted set stored at key.
     *
     * @param  mixed  $key
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zcard(mixed $key): Cluster|int|false {}

    /**
     * Returns the number of elements in the sorted set at key with a score between min and max.
     *
     * @param  mixed  $key
     * @param  mixed  $min
     * @param  mixed  $max
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zcount(mixed $key, mixed $min, mixed $max): Cluster|int|false {}

    /**
     * This command is similar to ZDIFFSTORE, but instead of storing the
     * resulting sorted set, it is returned to the client.
     *
     * @param  array  $keys
     * @param  array|null  $options
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zdiff(array $keys, array|null $options = null): Cluster|array|false {}

    /**
     * Computes the difference between the first and all successive
     * input sorted sets and stores the result in destination.
     *
     * @param  mixed  $dstkey
     * @param  array  $keys
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zdiffstore(mixed $dstkey, array $keys): Cluster|int|false {}

    /**
     * Increments the score of member in the sorted set stored at key by increment.
     *
     * @param  mixed  $key
     * @param  float  $score
     * @param  mixed  $member
     * @return Cluster|float|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zincrby(mixed $key, float $score, mixed $member): Cluster|float|false {}

    /**
     * This command is similar to ZINTERSTORE, but instead of storing
     * the resulting sorted set, it is returned to the client.
     *
     * @param  array  $keys
     * @param  array|null  $weights
     * @param  mixed  $options
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zinter(array $keys, array|null $weights = null, mixed $options = null): Cluster|array|false {}

    /**
     * Intersect multiple sorted sets and return the cardinality of the result.
     *
     * @param  array  $keys
     * @param  int  $limit
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zintercard(array $keys, int $limit = -1): Cluster|int|false {}

    /**
     * Computes the intersection of numkeys sorted sets given by the
     * specified keys, and stores the result in destination.
     *
     * @param  mixed  $dstkey
     * @param  array  $keys
     * @param  array|null  $weights
     * @param  mixed  $options
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zinterstore(mixed $dstkey, array $keys, array|null $weights = null, mixed $options = null): Cluster|int|false {}

    /**
     * When all the elements in a sorted set are inserted with the same score,
     * in order to force lexicographical ordering, this command returns the
     * number of elements in the sorted set at key with a value between min and max.
     *
     * @param  mixed  $key
     * @param  mixed  $min
     * @param  mixed  $max
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zlexcount(mixed $key, mixed $min, mixed $max): Cluster|int|false {}

    /**
     * Pops one or more elements, that are member-score pairs, from the
     * first non-empty sorted set in the provided list of key names.
     *
     * @param  array  $keys
     * @param  string  $from
     * @param  int  $count
     * @return Cluster|array|null|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zmpop(array $keys, string $from, int $count = 1): Cluster|array|null|false {}

    /**
     * Returns the scores associated with the specified members in the sorted set stored at key.
     *
     * @param  mixed  $key
     * @param  mixed  $members,...
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zmscore(mixed $key, mixed ...$members): Cluster|array|false {}

    /**
     * Removes and returns up to count members with the highest
     * scores in the sorted set stored at key.
     *
     * @param  mixed  $key
     * @param  int  $count
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zpopmax(mixed $key, int $count = 1): Cluster|array|false {}

    /**
     * Removes and returns up to count members with the lowest
     * scores in the sorted set stored at key.
     *
     * @param  mixed  $key
     * @param  int  $count
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zpopmin(mixed $key, int $count = 1): Cluster|array|false {}

    /**
     * When called with just the key argument, return a random element from the sorted set value stored at key.
     * If the provided count argument is positive, return an array of distinct elements.
     *
     * @param  mixed  $key
     * @param  array|null  $options
     * @return mixed
     */
    #[\Relay\Attributes\RedisCommand]
    public function zrandmember(mixed $key, array|null $options = null): mixed {}

    /**
     * Returns the specified range of elements in the sorted set stored at key.
     *
     * @param  mixed  $key
     * @param  string  $start
     * @param  string  $end
     * @param  mixed  $options
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zrange(mixed $key, string $start, string $end, mixed $options = null): Cluster|array|false {}

    /**
     * When all the elements in a sorted set are inserted with the same score,
     * in order to force lexicographical ordering, this command returns all
     * the elements in the sorted set at key with a value between min and max.
     *
     * @param  mixed  $key
     * @param  mixed  $min
     * @param  mixed  $max
     * @param  int  $offset
     * @param  int  $count
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zrangebylex(mixed $key, mixed $min, mixed $max, int $offset = -1, int $count = -1): Cluster|array|false {}

    /**
     * Returns all the elements in the sorted set at key with a score between
     * min and max (including elements with score equal to min or max).
     *
     * @param  mixed  $key
     * @param  mixed  $start
     * @param  mixed  $end
     * @param  mixed  $options
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zrangebyscore(mixed $key, mixed $start, mixed $end, mixed $options = null): Cluster|array|false {}

    /**
     * Returns all the elements in the sorted set at key with a score between
     * max and min (including elements with score equal to max or min).
     *
     * @param  mixed  $dstkey
     * @param  mixed  $srckey
     * @param  mixed  $start
     * @param  mixed  $end
     * @param  mixed  $options
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zrangestore(mixed $dstkey, mixed $srckey, mixed $start, mixed $end, mixed $options = null): Cluster|int|false {}

    /**
     * Returns the rank of member in the sorted set stored at key, with the scores
     * ordered from low to high. The rank (or index) is 0-based, which means
     * that the member with the lowest score has rank 0.
     *
     * @param  mixed  $key
     * @param  mixed  $rank
     * @param  bool  $withscore
     * @return Cluster|array|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zrank(mixed $key, mixed $rank, bool $withscore = false): Cluster|array|int|false {}

    /**
     * Removes the specified members from the sorted set stored at key.
     * Non existing members are ignored.
     *
     * @param  mixed  $key
     * @param  mixed  $args,...
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zrem(mixed $key, mixed ...$args): Cluster|int|false {}

    /**
     * When all the elements in a sorted set are inserted with the same score,
     * in order to force lexicographical ordering, this command removes all
     * elements in the sorted set stored at key between the
     * lexicographical range specified by min and max.
     *
     * @param  mixed  $key
     * @param  mixed  $min
     * @param  mixed  $max
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zremrangebylex(mixed $key, mixed $min, mixed $max): Cluster|int|false {}

    /**
     * Removes all elements in the sorted set stored at key with rank between
     * start and stop. Both start and stop are 0 -based indexes with 0 being
     * the element with the lowest score.
     *
     * @param  mixed  $key
     * @param  int  $start
     * @param  int  $end
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zremrangebyrank(mixed $key, int $start, int $end): Cluster|int|false {}

    /**
     * Removes all elements in the sorted set stored at key with
     * a score between min and max (inclusive).
     *
     * @param  mixed  $key
     * @param  mixed  $min
     * @param  mixed  $max
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zremrangebyscore(mixed $key, mixed $min, mixed $max): Cluster|int|false {}

    /**
     * Returns the specified range of elements in the sorted set stored at key.
     *
     * @param  mixed  $key
     * @param  int  $start
     * @param  int  $end
     * @param  mixed  $options
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zrevrange(mixed $key, int $start, int $end, mixed $options = null): Cluster|array|false {}

    /**
     * When all the elements in a sorted set are inserted with the same score,
     * in order to force lexicographical ordering, this command returns all
     * the elements in the sorted set at key with a value between max and min.
     *
     * @param  mixed  $key
     * @param  mixed  $max
     * @param  mixed  $min
     * @param  int  $offset
     * @param  int  $count
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zrevrangebylex(mixed $key, mixed $max, mixed $min, int $offset = -1, int $count = -1): Cluster|array|false {}

    /**
     * Returns all the elements in the sorted set at key with a score between
     * max and min (including elements with score equal to max or min).
     *
     * @param  mixed  $key
     * @param  mixed  $start
     * @param  mixed  $end
     * @param  mixed  $options
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zrevrangebyscore(mixed $key, mixed $start, mixed $end, mixed $options = null): Cluster|array|false {}

    /**
     * Returns the rank of member in the sorted set stored at key, with the scores
     * ordered from high to low. The rank (or index) is 0-based, which means
     * that the member with the highest score has rank 0.
     *
     * @param  mixed  $key
     * @param  mixed  $rank
     * @param  bool  $withscore
     * @return Cluster|array|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zrevrank(mixed $key, mixed $rank, bool $withscore = false): Cluster|array|int|false {}

    /**
     * Iterates elements of Sorted Set types and their associated scores.
     *
     * @param  mixed  $key
     * @param  mixed  $iterator
     * @param  mixed  $match
     * @param  int  $count
     * @return array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zscan(mixed $key, mixed &$iterator, mixed $match = null, int $count = 0): array|false {}

    /**
     * Returns the score of member in the sorted set at key.
     *
     * @param  mixed  $key
     * @param  mixed  $member
     * @return Cluster|float|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zscore(mixed $key, mixed $member): Cluster|float|false {}

    /**
     * This command is similar to ZUNIONSTORE, but instead of storing
     * the resulting sorted set, it is returned to the client.
     *
     * @param  array  $keys
     * @param  array|null  $weights
     * @param  mixed  $options
     * @return Cluster|array|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zunion(array $keys, array|null $weights = null, mixed $options = null): Cluster|array|false {}

    /**
     * Computes the union of numkeys sorted sets given by the
     * specified keys, and stores the result in destination.
     *
     * @param  mixed  $dstkey
     * @param  array  $keys
     * @param  array|null  $weights
     * @param  mixed  $options
     * @return Cluster|int|false
     */
    #[\Relay\Attributes\RedisCommand]
    public function zunionstore(mixed $dstkey, array $keys, array|null $weights = null, mixed $options = null): Cluster|int|false {}
}
