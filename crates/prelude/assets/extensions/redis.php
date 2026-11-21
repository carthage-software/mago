<?php

class Redis
{
    public const REDIS_NOT_FOUND = 0;
    public const REDIS_STRING = 1;
    public const REDIS_SET = 2;
    public const REDIS_LIST = 3;
    public const REDIS_ZSET = 4;
    public const REDIS_HASH = 5;
    public const REDIS_STREAM = 6;
    public const ATOMIC = 0;
    public const MULTI = 1;
    public const PIPELINE = 2;
    public const OPT_SERIALIZER = 1;
    public const OPT_PREFIX = 2;
    public const OPT_READ_TIMEOUT = 3;
    public const OPT_SCAN = 4;
    public const OPT_SLAVE_FAILOVER = 5;
    public const OPT_TCP_KEEPALIVE = 6;
    public const OPT_COMPRESSION = 7;
    public const OPT_REPLY_LITERAL = 8;
    public const OPT_COMPRESSION_LEVEL = 9;
    public const OPT_NULL_MULTIBULK_AS_NULL = 10;
    public const OPT_MAX_RETRIES = 11;
    public const OPT_BACKOFF_ALGORITHM = 12;
    public const OPT_BACKOFF_BASE = 13;
    public const OPT_BACKOFF_CAP = 14;
    public const SERIALIZER_NONE = 0;
    public const SERIALIZER_PHP = 1;
    public const SERIALIZER_IGBINARY = 2;
    public const SERIALIZER_MSGPACK = 3;
    public const SERIALIZER_JSON = 4;
    public const COMPRESSION_NONE = 0;
    public const COMPRESSION_LZF = 1;
    public const COMPRESSION_ZSTD = 2;
    public const COMPRESSION_ZSTD_DEFAULT = 3;
    public const COMPRESSION_ZSTD_MAX = 22;
    public const COMPRESSION_LZ4 = 3;
    public const SCAN_RETRY = 1;
    public const SCAN_NORETRY = 0;
    public const SCAN_PREFIX = 2;
    public const SCAN_NOPREFIX = 3;
    public const BEFORE = 'before';
    public const AFTER = 'after';
    public const LEFT = 'left';
    public const RIGHT = 'right';
    public const BACKOFF_ALGORITHM_DEFAULT = 0;
    public const BACKOFF_ALGORITHM_CONSTANT = 6;
    public const BACKOFF_ALGORITHM_UNIFORM = 5;
    public const BACKOFF_ALGORITHM_EXPONENTIAL = 4;
    public const BACKOFF_ALGORITHM_FULL_JITTER = 2;
    public const BACKOFF_ALGORITHM_EQUAL_JITTER = 3;
    public const BACKOFF_ALGORITHM_DECORRELATED_JITTER = 1;

    public function __construct(?array $options = null) {}

    public function __destruct() {}

    public function _compress(string $value): string {}

    public function _uncompress(string $value): string {}

    public function _prefix(string $key): string {}

    public function _serialize(mixed $value): string {}

    public function _unserialize(string $value): mixed {}

    public function _pack(mixed $value): string {}

    public function _unpack(string $value): mixed {}

    public function acl(string $subcmd, string ...$args): mixed {}

    public function append(string $key, mixed $value): Redis|int|false {}

    public function auth(mixed $credentials): Redis|bool {}

    public function bgSave(): Redis|bool {}

    public function bgrewriteaof(): Redis|bool {}

    public function bitcount(string $key, int $start = 0, int $end = -1, bool $bybit = false): Redis|int|false {}

    public function bitop(string $operation, string $deskey, string $srckey, string ...$other_keys): Redis|int|false {}

    public function bitpos(
        string $key,
        bool $bit,
        int $start = 0,
        int $end = -1,
        bool $bybit = false,
    ): Redis|int|false {}

    public function blPop(
        string|array $key_or_keys,
        string|float|int $timeout_or_key,
        mixed ...$extra_args,
    ): Redis|array|null|false {}

    public function brPop(
        string|array $key_or_keys,
        string|float|int $timeout_or_key,
        mixed ...$extra_args,
    ): Redis|array|null|false {}

    public function brpoplpush(string $src, string $dst, int|float $timeout): Redis|string|false {}

    public function bzPopMax(string|array $key, string|int $timeout_or_key, mixed ...$extra_args): Redis|array|false {}

    public function bzPopMin(string|array $key, string|int $timeout_or_key, mixed ...$extra_args): Redis|array|false {}

    public function clearLastError(): bool {}

    public function client(string $opt, mixed ...$args): mixed {}

    public function close(): bool {}

    public function command(string $opt = null, string|array $arg = null): mixed {}

    public function connect(
        string $host,
        int $port = 6379,
        float $timeout = 0,
        ?string $persistent_id = null,
        int $retry_interval = 0,
        float $read_timeout = 0,
        ?array $context = null,
    ): bool {}

    public function copy(string $src, string $dst, ?array $options = null): Redis|bool {}

    public function dbSize(): Redis|int|false {}

    public function debug(string $key): Redis|string {}

    public function decr(string $key, int $by = 1): Redis|int|false {}

    public function decrBy(string $key, int $value): Redis|int|false {}

    public function del(array|string $key, string ...$other_keys): Redis|int|false {}

    public function delete(array|string $key, string ...$other_keys): Redis|int|false {}

    public function discard(): Redis|bool {}

    public function dump(string $key): Redis|string|false {}

    public function echo(string $str): Redis|string|false {}

    public function eval(string $script, array $args = [], int $num_keys = 0): mixed {}

    public function eval_ro(string $script_sha, array $args = [], int $num_keys = 0): mixed {}

    public function evalsha(string $script_sha, array $args = [], int $num_keys = 0): mixed {}

    public function evalsha_ro(string $script_sha, array $args = [], int $num_keys = 0): mixed {}

    public function exec(): Redis|array|false {}

    public function exists(mixed $key, mixed ...$other_keys): Redis|int|bool {}

    public function expire(string $key, int $timeout, ?string $mode = null): Redis|bool {}

    public function expireAt(string $key, int $timestamp, ?string $mode = null): Redis|bool {}

    public function expiretime(string $key): Redis|int|false {}

    public function failover(?array $to = null, bool $abort = false, int $timeout = 0): Redis|bool {}

    public function flushAll(?bool $async = null): Redis|bool {}

    public function flushDB(?bool $async = null): Redis|bool {}

    public function get(string $key): mixed {}

    public function getAuth(): mixed {}

    public function getBit(string $key, int $idx): Redis|int|false {}

    public function getDBNum(): int {}

    public function getDel(string $key): Redis|string|bool {}

    public function getEx(string $key, ?array $options = null): Redis|string|bool {}

    public function getHost(): string {}

    public function getLastError(): ?string {}

    public function getMode(): int {}

    public function getOption(int $option): mixed {}

    public function getPersistentID(): ?string {}

    public function getPort(): int {}

    public function getRange(string $key, int $start, int $end): Redis|string|false {}

    public function getReadTimeout(): float {}

    public function getSet(string $key, mixed $value): Redis|string|false {}

    public function getTimeout(): float|false {}

    public function getTransferredBytes(): array {}

    public function hDel(string $key, string $field, string ...$other_fields): Redis|int|false {}

    public function hExists(string $key, string $field): Redis|bool {}

    public function hGet(string $key, string $field): mixed {}

    public function hGetAll(string $key): Redis|array|false {}

    public function hIncrBy(string $key, string $field, int $value): Redis|int|false {}

    public function hIncrByFloat(string $key, string $field, float $value): Redis|float|false {}

    public function hKeys(string $key): Redis|array|false {}

    public function hLen(string $key): Redis|int|false {}

    public function hMGet(string $key, array $fields): Redis|array|false {}

    public function hMSet(string $key, array $fieldvals): Redis|bool {}

    public function hSet(string $key, mixed ...$fields_and_vals): Redis|int|false {}

    public function hSetNx(string $key, string $field, mixed $value): Redis|bool {}

    public function hStrLen(string $key, string $field): Redis|int|false {}

    public function hVals(string $key): Redis|array|false {}

    public function incr(string $key, int $by = 1): Redis|int|false {}

    public function incrBy(string $key, int $value): Redis|int|false {}

    public function incrByFloat(string $key, float $value): Redis|float|false {}

    public function info(string ...$sections): Redis|array|false {}

    public function isConnected(): bool {}

    public function keys(string $pattern): Redis|array|false {}

    public function lGet(string $key, int $index): mixed {}

    public function lIndex(string $key, int $index): mixed {}

    public function lInsert(string $key, string $pos, mixed $pivot, mixed $value): Redis|int|false {}

    public function lLen(string $key): Redis|int|false {}

    public function lMove(string $src, string $dst, string $wherefrom, string $whereto): Redis|string|false {}

    public function lPop(string $key, int $count = 0): Redis|bool|string|array {}

    public function lPos(string $key, mixed $value, ?array $options = null): Redis|null|bool|int|array {}

    public function lPush(string $key, mixed ...$elements): Redis|int|false {}

    public function lPushx(string $key, mixed $value): Redis|int|false {}

    public function lRange(string $key, int $start, int $end): Redis|array|false {}

    public function lRem(string $key, mixed $value, int $count = 0): Redis|int|false {}

    public function lSet(string $key, int $index, mixed $value): Redis|bool {}

    public function lTrim(string $key, int $start, int $end): Redis|bool {}

    public function lastSave(): int {}

    public function mget(array $keys): Redis|array {}

    public function migrate(
        string $host,
        int $port,
        string|array $key,
        int $dstdb,
        int $timeout,
        bool $copy = false,
        bool $replace = false,
        mixed $credentials = null,
    ): Redis|bool {}

    public function move(string $key, int $index): Redis|bool {}

    public function mset(array $key_values): Redis|bool {}

    public function msetnx(array $key_values): Redis|bool {}

    public function multi(int $value = Redis::MULTI): Redis|bool {}

    public function object(string $subcommand, string $key = ''): mixed {}

    public function open(
        string $host,
        int $port = 6379,
        float $timeout = 0,
        ?string $persistent_id = null,
        int $retry_interval = 0,
        float $read_timeout = 0,
        ?array $context = null,
    ): bool {}

    public function pconnect(
        string $host,
        int $port = 6379,
        float $timeout = 0,
        ?string $persistent_id = null,
        int $retry_interval = 0,
        float $read_timeout = 0,
        ?array $context = null,
    ): bool {}

    public function persist(string $key): Redis|bool {}

    public function pexpire(string $key, int $timeout, ?string $mode = null): Redis|bool {}

    public function pexpireAt(string $key, int $timestamp, ?string $mode = null): Redis|bool {}

    public function pexpiretime(string $key): Redis|int|false {}

    public function pfadd(string $key, array $elements): Redis|int {}

    public function pfcount(string $key): Redis|int|false {}

    public function pfmerge(string $dst, array $srckeys): Redis|bool {}

    public function ping(?string $message = null): Redis|string|bool {}

    public function pipeline(): Redis|bool {}

    public function popen(
        string $host,
        int $port = 6379,
        float $timeout = 0,
        ?string $persistent_id = null,
        int $retry_interval = 0,
        float $read_timeout = 0,
        ?array $context = null,
    ): bool {}

    public function psetex(string $key, int $expire, mixed $value): Redis|bool {}

    public function psubscribe(array $patterns, callable $cb): bool {}

    public function pttl(string $key): Redis|int|false {}

    public function publish(string $channel, string $message): Redis|int|false {}

    public function pubsub(string $command, mixed $arg = null): mixed {}

    public function punsubscribe(array $patterns): Redis|array|bool {}

    public function rPop(string $key, int $count = 0): Redis|array|string|bool {}

    public function rpoplpush(string $srckey, string $dstkey): Redis|string|false {}

    public function rPush(string $key, mixed ...$elements): Redis|int|false {}

    public function rPushx(string $key, mixed $value): Redis|int|false {}

    public function randomKey(): Redis|string|false {}

    public function rawcommand(string $command, mixed ...$args): mixed {}

    public function rename(string $key_src, string $key_dst): Redis|bool {}

    public function renameNx(string $key_src, string $key_dst): Redis|bool {}

    public function reset(): Redis|bool {}

    public function restore(string $key, int $ttl, string $value, ?array $options = null): Redis|bool {}

    public function role(): mixed {}

    public function sAdd(string $key, mixed $value, mixed ...$other_values): Redis|int|false {}

    public function sAddArray(string $key, array $values): Redis|int|false {}

    public function sDiff(string $key, string ...$other_keys): Redis|array|false {}

    public function sDiffStore(string $dst, string $key, string ...$other_keys): Redis|int|false {}

    public function sInter(array|string $key, string ...$other_keys): Redis|array|false {}

    public function sInterStore(array|string $dst, string ...$other_keys): Redis|int|false {}

    public function sMembers(string $key): Redis|array|false {}

    public function sMisMember(string $key, string $member, string ...$other_members): Redis|array|false {}

    public function sMove(string $src, string $dst, mixed $value): Redis|bool {}

    public function sPop(string $key, int $count = 0): Redis|string|array|false {}

    public function sRandMember(string $key, int $count = 0): Redis|string|array|false {}

    public function sRem(string $key, mixed $value, mixed ...$other_values): Redis|int|false {}

    public function sUnion(string $key, string ...$other_keys): Redis|array|false {}

    public function sUnionStore(string $dst, string $key, string ...$other_keys): Redis|int|false {}

    public function save(): Redis|bool {}

    /**
     * @param-out int $iterator
     */
    public function scan(?int &$iterator, ?string $pattern = null, int $count = 0, ?string $type = null): array|false {}

    public function scard(string $key): Redis|int|false {}

    public function script(string $command, mixed ...$args): mixed {}

    public function select(int $db): Redis|bool {}

    public function set(string $key, mixed $value, mixed $options = null): Redis|string|bool {}

    public function setBit(string $key, int $idx, bool $value): Redis|int|false {}

    public function setOption(int $option, mixed $value): bool {}

    public function setRange(string $key, int $index, string $value): Redis|int|false {}

    public function setex(string $key, int $expire, mixed $value): Redis|bool {}

    public function setnx(string $key, mixed $value): Redis|bool {}

    public function sintercard(array $keys, int $limit = -1): Redis|int|false {}

    public function sismember(string $key, mixed $value): Redis|bool {}

    public function slaveof(?string $host = null, int $port = 6379): Redis|bool {}

    public function slowlog(string $mode, int $option = 0): mixed {}

    public function sort(string $key, ?array $options = null): mixed {}

    public function sortAsc(
        string $key,
        ?string $pattern = null,
        mixed $get = null,
        int $offset = -1,
        int $count = -1,
        ?string $store = null,
    ): array {}

    public function sortAscAlpha(
        string $key,
        ?string $pattern = null,
        mixed $get = null,
        int $offset = -1,
        int $count = -1,
        ?string $store = null,
    ): array {}

    public function sortDesc(
        string $key,
        ?string $pattern = null,
        mixed $get = null,
        int $offset = -1,
        int $count = -1,
        ?string $store = null,
    ): array {}

    public function sortDescAlpha(
        string $key,
        ?string $pattern = null,
        mixed $get = null,
        int $offset = -1,
        int $count = -1,
        ?string $store = null,
    ): array {}

    public function sort_ro(string $key, ?array $options = null): mixed {}

    /**
     * @param-out int $iterator
     */
    public function sscan(string $key, ?int &$iterator, ?string $pattern = null, int $count = 0): array|false {}

    public function strlen(string $key): Redis|int|false {}

    public function subscribe(array $channels, callable $cb): bool {}

    public function time(): Redis|array {}

    public function ttl(string $key): Redis|int|false {}

    public function type(string $key): Redis|int|false {}

    public function unlink(array|string $key, string ...$other_keys): Redis|int|false {}

    public function unsubscribe(array $channels): Redis|array|bool {}

    public function unwatch(): Redis|bool {}

    public function wait(int $numreplicas, int $timeout): int|false {}

    public function watch(string|array $key, string ...$other_keys): Redis|bool {}

    public function xack(string $key, string $group, array $ids): Redis|int|false {}

    public function xadd(
        string $key,
        string $id,
        array $values,
        int $maxlen = 0,
        bool $approx = false,
        bool $nomkstream = false,
    ): Redis|string|false {}

    public function xautoclaim(
        string $key,
        string $group,
        string $consumer,
        int $min_idle,
        string $start,
        int $count = -1,
        bool $justid = false,
    ): Redis|array|bool {}

    public function xclaim(
        string $key,
        string $group,
        string $consumer,
        int $min_idle,
        array $ids,
        array $options = [],
    ): Redis|array|false {}

    public function xdel(string $key, array $ids): Redis|int|false {}

    public function xgroup(
        string $operation,
        ?string $key = null,
        ?string $group = null,
        ?string $id_or_consumer = null,
        bool $mkstream = false,
        int $entries_read = -2,
    ): mixed {}

    public function xinfo(string $operation, ?string $arg1 = null, ?string $arg2 = null, int $count = -1): mixed {}

    public function xlen(string $key): Redis|int|false {}

    public function xpending(
        string $key,
        string $group,
        ?string $start = null,
        ?string $end = null,
        int $count = -1,
        ?string $consumer = null,
    ): Redis|array|false {}

    public function xrange(string $key, string $start, string $end, int $count = -1): Redis|array|bool {}

    public function xread(array $streams, int $count = -1, int $block = -1): Redis|array|bool {}

    public function xreadgroup(
        string $group,
        string $consumer,
        array $streams,
        int $count = 1,
        int $block = 1,
    ): Redis|array|bool {}

    public function xrevrange(string $key, string $start, string $end, int $count = -1): Redis|array|bool {}

    public function xsetid(
        string $key,
        string $id,
        ?int $entries_added = null,
        ?string $max_deleted_id = null,
    ): Redis|bool {}

    public function xtrim(
        string $key,
        string $threshold,
        bool $approx = false,
        bool $minid = false,
        int $limit = -1,
    ): Redis|int|false {}

    public function zAdd(
        string $key,
        array|float $score_or_options,
        mixed ...$more_scores_and_mems,
    ): Redis|int|float|false {}

    public function zCard(string $key): Redis|int|false {}

    public function zCount(string $key, string $start, string $end): Redis|int|false {}

    public function zIncrBy(string $key, float $value, mixed $member): Redis|float|false {}

    public function zLexCount(string $key, string $min, string $max): Redis|int|false {}

    public function zMscore(string $key, string $member, string ...$other_members): Redis|array|false {}

    public function zPopMax(string $key, int $count = null): Redis|array|false {}

    public function zPopMin(string $key, int $count = null): Redis|array|false {}

    public function zRange(string $key, mixed $start, mixed $end, array|bool|null $options = null): Redis|array|false {}

    public function zRangeByLex(
        string $key,
        string $min,
        string $max,
        int $offset = -1,
        int $count = -1,
    ): Redis|array|false {}

    public function zRangeByScore(string $key, string $start, string $end, array $options = []): Redis|array|false {}

    public function zRangeStore(
        string $dstkey,
        string $srckey,
        string $start,
        string $end,
        array|bool|null $options = null,
    ): Redis|int|false {}

    public function zRandMember(string $key, ?array $options = null): Redis|string|array {}

    public function zRank(string $key, mixed $member): Redis|int|false {}

    public function zRem(string $key, mixed $member, mixed ...$other_members): Redis|int|false {}

    public function zRemRangeByLex(string $key, string $min, string $max): Redis|int|false {}

    public function zRemRangeByRank(string $key, int $start, int $end): Redis|int|false {}

    public function zRemRangeByScore(string $key, string $start, string $end): Redis|int|false {}

    public function zRevRange(string $key, int $start, int $end, ?array $options = null): Redis|array|false {}

    public function zRevRangeByLex(
        string $key,
        string $max,
        string $min,
        int $offset = -1,
        int $count = -1,
    ): Redis|array|false {}

    public function zRevRangeByScore(string $key, string $start, string $end, array $options = []): Redis|array|false {}

    public function zRevRank(string $key, mixed $member): Redis|int|false {}

    /**
     * @param-out int $iterator
     */
    public function zscan(string $key, ?int &$iterator, ?string $pattern = null, int $count = 0): Redis|array|false {}

    public function zScore(string $key, mixed $member): Redis|float|false {}

    public function zdiff(array $keys, array $options = null): Redis|array|false {}

    public function zdiffstore(string $dst, array $keys): Redis|int|false {}

    public function zinter(array $keys, ?array $weights = null, ?array $options = null): Redis|array|false {}

    public function zintercard(array $keys, int $limit = -1): Redis|int|false {}

    public function zinterstore(
        string $dst,
        array $keys,
        ?array $weights = null,
        ?string $aggregate = null,
    ): Redis|int|false {}

    public function zunion(array $keys, ?array $weights = null, ?array $options = null): Redis|array|false {}

    public function zunionstore(
        string $dst,
        array $keys,
        ?array $weights = null,
        ?string $aggregate = null,
    ): Redis|int|false {}
}

class RedisException extends Exception {}

class RedisArray
{
    public function __construct(string $name, ?array $hosts = null, ?array $options = null) {}

    public function __call(string $name, array $argv): mixed {}

    public function _hosts(): array {}

    public function _target(string $key): string {}

    public function _instance(string $host): Redis {}

    public function _function(): ?callable {}

    public function _distributor(): ?callable {}

    public function _rehash(?callable $fn = null): bool {}

    public function keys(string $pattern): array|false {}

    public function save(): bool {}

    public function bgsave(): bool {}

    public function getOption(int $option): array {}

    public function setOption(int $option, string $value): array {}

    public function select(int $index): bool {}

    public function info(): array|false {}

    public function ping(): bool {}

    public function flushDB(?bool $async = null): bool {}

    public function flushAll(?bool $async = null): bool {}

    public function mget(array $keys): array {}

    public function mset(array $pairs): bool {}

    public function del(string ...$keys): int|false {}

    public function unlink(string ...$keys): int|false {}

    public function multi(Redis $host): Redis {}

    public function exec(): mixed {}

    public function discard(): bool {}
}

class RedisSentinel
{
    public function __construct(mixed $options) {}

    public function ckquorum(string $master): bool {}

    public function failover(string $master): bool {}

    public function flushconfig(): bool {}

    public function getMasterAddrByName(string $master): array|false {}

    public function master(string $master): array|false {}

    public function masters(): array|false {}

    public function myid(): string|false {}

    public function ping(): bool {}

    public function reset(string $pattern): bool {}

    public function sentinels(string $master): array|false {}

    public function slaves(string $master): array|false {}
}

class RedisCluster
{
    public const OPT_SLAVE_FAILOVER = 5;

    public const FAILOVER_NONE = 0;

    public const FAILOVER_ERROR = 1;

    public const FAILOVER_DISTRIBUTE = 2;

    public const FAILOVER_DISTRIBUTE_SLAVES = 3;

    public function __construct(
        ?string $name,
        ?array $seeds = null,
        int|float $timeout = 0,
        int|float $read_timeout = 0,
        bool $persistent = false,
        #[\SensitiveParameter]
        mixed $auth = null,
        ?array $context = null,
    ) {}

    public function _compress(string $value): string {}

    public function _uncompress(string $value): string {}

    public function _serialize(mixed $value): bool|string {}

    public function _unserialize(string $value): mixed {}

    public function _pack(mixed $value): string {}

    public function _digest(mixed $value): string {}

    public function _unpack(string $value): mixed {}

    public function _prefix(string $key): bool|string {}

    public function _masters(): array {}

    public function _redir(): ?string {}

    public function acl(string|array $key_or_address, string $subcmd, string ...$args): mixed {}

    public function append(string $key, mixed $value): RedisCluster|bool|int {}

    public function bgrewriteaof(string|array $key_or_address): RedisCluster|bool {}

    public function wait(string|array $key_or_address, int $numreplicas, int $timeout): RedisCluster|int|false {}

    public function waitaof(
        string|array $key_or_address,
        int $numlocal,
        int $numreplicas,
        int $timeout,
    ): RedisCluster|array|false {}

    public function bgsave(string|array $key_or_address): RedisCluster|bool {}

    public function bitcount(string $key, int $start = 0, int $end = -1, bool $bybit = false): RedisCluster|bool|int {}

    public function bitop(
        string $operation,
        string $deskey,
        string $srckey,
        string ...$otherkeys,
    ): RedisCluster|bool|int {}

    public function bitpos(
        string $key,
        bool $bit,
        int $start = 0,
        int $end = -1,
        bool $bybit = false,
    ): RedisCluster|int|false {}

    public function bitrangeintarray(
        string $key,
        int $start,
        int $end,
        ?array $options = null,
    ): RedisCluster|array|false {}

    public function bless(
        string $command,
        string|int|null $key = null,
        ?string $directive = null,
        ?array $options = null,
        array|string|null $key_or_address = null,
    ): mixed {}

    public function blpop(
        string|array $key,
        string|float|int $timeout_or_key,
        mixed ...$extra_args,
    ): RedisCluster|array|false|null {}

    public function brpop(
        string|array $key,
        string|float|int $timeout_or_key,
        mixed ...$extra_args,
    ): RedisCluster|array|false|null {}

    public function brpoplpush(string $srckey, string $deskey, int $timeout): mixed {}

    public function lmove(string $src, string $dst, string $wherefrom, string $whereto): RedisCluster|string|false {}

    public function lmovem(
        string $src,
        string $dst,
        string $wherefrom,
        string $whereto,
        ?array $options = null,
    ): RedisCluster|array|false {}

    public function blmovem(
        string $src,
        string $dst,
        string $wherefrom,
        string $whereto,
        float $timeout,
        ?array $options = null,
    ): RedisCluster|array|false {}

    public function blmove(
        string $src,
        string $dst,
        string $wherefrom,
        string $whereto,
        float $timeout,
    ): RedisCluster|string|false {}

    public function bzpopmax(string|array $key, string|int $timeout_or_key, mixed ...$extra_args): array {}

    public function bzpopmin(string|array $key, string|int $timeout_or_key, mixed ...$extra_args): array {}

    public function bzmpop(float $timeout, array $keys, string $from, int $count = 1): RedisCluster|array|false|null {}

    public function zmpop(array $keys, string $from, int $count = 1): RedisCluster|array|false|null {}

    public function blmpop(float $timeout, array $keys, string $from, int $count = 1): RedisCluster|array|false|null {}

    public function lmpop(array $keys, string $from, int $count = 1): RedisCluster|array|false|null {}

    public function clearlasterror(): bool {}

    public function client(string|array $key_or_address, string $subcommand, ?string $arg = null): array|string|bool {}

    public function close(): bool {}

    public function reconnect(): bool {}

    public function cluster(string|array $key_or_address, string $command, mixed ...$extra_args): mixed {}

    public function command(mixed ...$extra_args): mixed {}

    public function config(string|array $key_or_address, string $subcommand, mixed ...$extra_args): mixed {}

    public function dbsize(string|array $key_or_address): RedisCluster|int {}

    public function copy(string $src, string $dst, ?array $options = null): RedisCluster|bool {}

    public function decr(string $key, int $by = 1): RedisCluster|int|false {}

    public function decrby(string $key, int $value): RedisCluster|int|false {}

    public function decrbyfloat(string $key, float $value): float {}

    public function del(array|string $key, string ...$other_keys): RedisCluster|int|false {}

    public function delex(string $key, ?array $options = null): RedisCluster|int|false {}

    public function delifeq(string $key, mixed $value): RedisCluster|int|false {}

    public function discard(): bool {}

    public function dump(string $key): RedisCluster|string|false {}

    public function echo(string|array $key_or_address, string $msg): RedisCluster|string|false {}

    public function eval(string $script, array $args = [], int $num_keys = 0): mixed {}

    public function eval_ro(string $script, array $args = [], int $num_keys = 0): mixed {}

    public function evalsha(string $script_sha, array $args = [], int $num_keys = 0): mixed {}

    public function evalsha_ro(string $script_sha, array $args = [], int $num_keys = 0): mixed {}

    public function exec(): array|false {}

    public function exists(mixed $key, mixed ...$other_keys): RedisCluster|int|bool {}

    public function touch(mixed $key, mixed ...$other_keys): RedisCluster|int|bool {}

    public function expire(string $key, int $timeout, ?string $mode = null): RedisCluster|bool {}

    public function expireat(string $key, int $timestamp, ?string $mode = null): RedisCluster|bool {}

    public function expiretime(string $key): RedisCluster|int|false {}

    public function pexpiretime(string $key): RedisCluster|int|false {}

    public function flushall(string|array $key_or_address, bool $async = false): RedisCluster|bool {}

    public function flushdb(string|array $key_or_address, bool $async = false): RedisCluster|bool {}

    public function geoadd(
        string $key,
        float $lng,
        float $lat,
        string $member,
        mixed ...$other_triples_and_options,
    ): RedisCluster|int|false {}

    public function geodist(string $key, string $src, string $dest, ?string $unit = null): RedisCluster|float|false {}

    public function geohash(string $key, string $member, string ...$other_members): RedisCluster|array|false {}

    public function geopos(string $key, string $member, string ...$other_members): RedisCluster|array|false {}

    public function georadius(
        string $key,
        float $lng,
        float $lat,
        float $radius,
        string $unit,
        array $options = [],
    ): mixed {}

    public function georadius_ro(
        string $key,
        float $lng,
        float $lat,
        float $radius,
        string $unit,
        array $options = [],
    ): mixed {}

    public function georadiusbymember(
        string $key,
        string $member,
        float $radius,
        string $unit,
        array $options = [],
    ): mixed {}

    public function georadiusbymember_ro(
        string $key,
        string $member,
        float $radius,
        string $unit,
        array $options = [],
    ): mixed {}

    public function geosearch(
        string $key,
        array|string $position,
        array|int|float $shape,
        string $unit,
        array $options = [],
    ): RedisCluster|array {}

    public function geosearchstore(
        string $dst,
        string $src,
        array|string $position,
        array|int|float $shape,
        string $unit,
        array $options = [],
    ): RedisCluster|array|int|false {}

    public function get(string $key): mixed {}

    public function getdel(string $key): mixed {}

    public function getWithMeta(string $key): RedisCluster|array|false {}

    public function getex(string $key, array $options = []): RedisCluster|string|false {}

    public function getbit(string $key, int $value): RedisCluster|int|false {}

    public function getlasterror(): ?string {}

    public function getmode(): int {}

    public function getoption(int $option): mixed {}

    public function getrange(string $key, int $start, int $end): RedisCluster|string|false {}

    public function lcs(string $key1, string $key2, ?array $options = null): RedisCluster|string|array|int|false {}

    public function getset(string $key, mixed $value): RedisCluster|string|bool {}

    public function gettransferredbytes(): array|false {}

    public function cleartransferredbytes(): void {}

    public function hdel(string $key, string $member, string ...$other_members): RedisCluster|int|false {}

    public function hexists(string $key, string $member): RedisCluster|bool {}

    public function hget(string $key, string $member): mixed {}

    public function hgetall(string $key): RedisCluster|array|false {}

    public function hgetWithMeta(string $key, string $member): mixed {}

    public function hincrby(string $key, string $member, int $value): RedisCluster|int|false {}

    public function hincrbyfloat(string $key, string $member, float $value): RedisCluster|float|false {}

    public function hkeys(string $key): RedisCluster|array|false {}

    public function hlen(string $key): RedisCluster|int|false {}

    public function hmget(string $key, array $keys): RedisCluster|array|false {}

    public function hgetex(string $key, array $fields, string|array|null $expiry = null): RedisCluster|array|false {}

    public function hsetex(string $key, array $fields, ?array $expiry = null): RedisCluster|int|false {}

    public function hgetdel(string $key, array $fields): RedisCluster|array|false {}

    public function himport(
        string $op,
        string $hash,
        ?string $fieldset = null,
        array $fields = [],
    ): RedisCluster|bool|int {}

    public function hmset(string $key, array $key_values): RedisCluster|bool {}

    public function hscan(
        string $key,
        int|string|null &$iterator,
        ?string $pattern = null,
        int $count = 0,
    ): array|bool {}

    public function expiremember(string $key, string $field, int $ttl, ?string $unit = null): RedisCluster|int|false {}

    public function expirememberat(string $key, string $field, int $timestamp): RedisCluster|int|false {}

    public function hrandfield(string $key, ?array $options = null): RedisCluster|string|array {}

    public function hset(string $key, string $member, mixed $value): RedisCluster|int|false {}

    public function hsetnx(string $key, string $member, mixed $value): RedisCluster|bool {}

    public function hstrlen(string $key, string $field): RedisCluster|int|false {}

    public function hexpire(string $key, int $ttl, array $fields, ?string $mode = null): RedisCluster|array|false {}

    public function hpexpire(string $key, int $ttl, array $fields, ?string $mode = null): RedisCluster|array|false {}

    public function hexpireat(string $key, int $time, array $fields, ?string $mode = null): RedisCluster|array|false {}

    public function hpexpireat(
        string $key,
        int $mstime,
        array $fields,
        ?string $mode = null,
    ): RedisCluster|array|false {}

    public function httl(string $key, array $fields): RedisCluster|array|false {}

    public function hpttl(string $key, array $fields): RedisCluster|array|false {}

    public function hexpiretime(string $key, array $fields): RedisCluster|array|false {}

    public function hpexpiretime(string $key, array $fields): RedisCluster|array|false {}

    public function hpersist(string $key, array $fields): RedisCluster|array|false {}

    public function hvals(string $key): RedisCluster|array|false {}

    public function incr(string $key, int $by = 1): RedisCluster|int|false {}

    public function incrby(string $key, int $value): RedisCluster|int|false {}

    public function incrbyfloat(string $key, float $value): RedisCluster|float|false {}

    public function increx(
        string $key,
        int|float|null $value = null,
        ?array $options = null,
    ): RedisCluster|array|false {}

    public function info(string|array $key_or_address, string ...$sections): RedisCluster|array|false {}

    public function keys(string $pattern): RedisCluster|array|false {}

    public function lastsave(string|array $key_or_address): RedisCluster|int|false {}

    public function lget(string $key, int $index): RedisCluster|string|bool {}

    public function lindex(string $key, int $index): mixed {}

    public function linsert(string $key, string $pos, mixed $pivot, mixed $value): RedisCluster|int|false {}

    public function llen(string $key): RedisCluster|int|bool {}

    public function lpop(string $key, int $count = 0): RedisCluster|bool|string|array {}

    public function lpos(string $key, mixed $value, ?array $options = null): RedisCluster|bool|int|array|null {}

    public function lpush(string $key, mixed $value, mixed ...$other_values): RedisCluster|int|bool {}

    public function lpushx(string $key, mixed $value): RedisCluster|int|bool {}

    public function lrange(string $key, int $start, int $end): RedisCluster|array|false {}

    public function lrem(string $key, mixed $value, int $count = 0): RedisCluster|int|bool {}

    public function lset(string $key, int $index, mixed $value): RedisCluster|bool {}

    public function ltrim(string $key, int $start, int $end): RedisCluster|bool {}

    public function mget(array $keys): RedisCluster|array|false {}

    public function mset(array $key_values): RedisCluster|bool {}

    public function msetnx(array $key_values): RedisCluster|array|false {}

    public function msetex(array $key_vals, int|float|array|null $expiry = null): RedisCluster|int|false {}

    /* We only support Redis::MULTI in RedisCluster but take the argument
     * so we can test MULTI..EXEC with RedisTest.php and in the event
     * we add pipeline support in the future. */
    public function multi(int $value = Redis::MULTI): RedisCluster|bool {}

    public function object(string $subcommand, string $key): RedisCluster|int|string|false {}

    public function persist(string $key): RedisCluster|bool {}

    public function pexpire(string $key, int $timeout, ?string $mode = null): RedisCluster|bool {}

    public function pexpireat(string $key, int $timestamp, ?string $mode = null): RedisCluster|bool {}

    public function pfadd(string $key, array $elements): RedisCluster|bool {}

    public function pfcount(string $key): RedisCluster|int|false {}

    public function pfmerge(string $key, array $keys): RedisCluster|bool {}

    public function ping(string|array $key_or_address, ?string $message = null): mixed {}

    public function psetex(string $key, int $timeout, string $value): RedisCluster|bool {}

    public function psubscribe(array $patterns, callable $callback): void {}

    public function pttl(string $key): RedisCluster|int|false {}

    public function publish(string $channel, string $message): RedisCluster|bool|int {}

    public function pubsub(string|array $key_or_address, string ...$values): mixed {}

    public function punsubscribe(string $pattern, string ...$other_patterns): bool|array {}

    public function randomkey(string|array $key_or_address): RedisCluster|bool|string {}

    public function rawcommand(string|array $key_or_address, string $command, mixed ...$args): mixed {}

    public function rename(string $key_src, string $key_dst): RedisCluster|bool {}

    public function renamenx(string $key, string $newkey): RedisCluster|bool {}

    public function restore(string $key, int $timeout, string $value, ?array $options = null): RedisCluster|bool {}

    public function role(string|array $key_or_address): mixed {}

    public function rpop(string $key, int $count = 0): RedisCluster|bool|string|array {}

    public function rpoplpush(string $src, string $dst): RedisCluster|bool|string {}

    public function rpush(string $key, mixed ...$elements): RedisCluster|int|false {}

    public function rpushx(string $key, string $value): RedisCluster|bool|int {}

    public function sadd(string $key, mixed $value, mixed ...$other_values): RedisCluster|int|false {}

    public function saddarray(string $key, array $values): RedisCluster|bool|int {}

    public function save(string|array $key_or_address): RedisCluster|bool {}

    public function scan(
        int|string|null &$iterator,
        string|array $key_or_address,
        ?string $pattern = null,
        int $count = 0,
    ): bool|array {}

    public function scard(string $key): RedisCluster|int|false {}

    public function script(string|array $key_or_address, mixed ...$args): mixed {}

    public function sdiff(string $key, string ...$other_keys): RedisCluster|array|false {}

    public function sdiffstore(string $dst, string $key, string ...$other_keys): RedisCluster|int|false {}

    public function set(string $key, mixed $value, mixed $options = null): RedisCluster|string|bool {}

    public function setbit(string $key, int $offset, bool $onoff): RedisCluster|int|false {}

    public function setex(string $key, int $expire, mixed $value): RedisCluster|bool {}

    public function setnx(string $key, mixed $value): RedisCluster|bool {}

    public function setoption(int $option, mixed $value): bool {}

    public function setrange(string $key, int $offset, string $value): RedisCluster|int|false {}

    public function sinter(array|string $key, string ...$other_keys): RedisCluster|array|false {}

    public function sintercard(array $keys, int $limit = -1): RedisCluster|int|false {}

    public function sunioncard(array $keys, ?array $options = null): RedisCluster|int|false {}

    public function sdiffcard(array $keys, ?array $options = null): RedisCluster|int|false {}

    public function sinterstore(array|string $key, string ...$other_keys): RedisCluster|int|false {}

    public function sismember(string $key, mixed $value): RedisCluster|bool {}

    public function smismember(string $key, string $member, string ...$other_members): RedisCluster|array|false {}

    public function slowlog(string|array $key_or_address, mixed ...$args): mixed {}

    public function smembers(string $key): RedisCluster|array|false {}

    public function smove(string $src, string $dst, string $member): RedisCluster|bool {}

    public function sort(string $key, ?array $options = null): RedisCluster|array|bool|int|string {}

    public function sort_ro(string $key, ?array $options = null): RedisCluster|array|bool|int|string {}

    public function spop(string $key, int $count = 0): RedisCluster|string|array|false {}

    public function srandmember(string $key, int $count = 0): RedisCluster|string|array|false {}

    public function srem(string $key, mixed $value, mixed ...$other_values): RedisCluster|int|false {}

    public function sscan(
        string $key,
        int|string|null &$iterator,
        ?string $pattern = null,
        int $count = 0,
    ): array|false {}

    public function strlen(string $key): RedisCluster|int|false {}

    public function subscribe(array $channels, callable $cb): void {}

    public function sunion(string $key, string ...$other_keys): RedisCluster|bool|array {}

    public function sunionstore(string $dst, string $key, string ...$other_keys): RedisCluster|int|false {}

    public function time(string|array $key_or_address): RedisCluster|bool|array {}

    public function ttl(string $key): RedisCluster|int|false {}

    public function type(string $key): RedisCluster|int|false {}

    public function unsubscribe(array $channels): bool|array {}

    public function unlink(array|string $key, string ...$other_keys): RedisCluster|int|false {}

    public function unwatch(): bool {}

    public function watch(string $key, string ...$other_keys): RedisCluster|bool {}

    public function vadd(string $key, array $values, mixed $element, ?array $options = null): RedisCluster|int|false {}

    public function vsim(string $key, mixed $member, ?array $options = null): RedisCluster|array|false {}

    public function vcard(string $key): RedisCluster|int|false {}

    public function vdim(string $key): RedisCluster|int|false {}

    public function vinfo(string $key): RedisCluster|array|false {}

    public function vismember(string $key, mixed $member): RedisCluster|bool {}

    public function vemb(string $key, mixed $member, bool $raw = false): RedisCluster|array|false {}

    public function vrandmember(string $key, int $count = 0): RedisCluster|array|string|false {}

    public function vrange(string $key, string $min, string $max, int $count = -1): RedisCluster|array|false {}

    public function vrem(string $key, mixed $member): RedisCluster|int|false {}

    public function vlinks(string $key, mixed $member, bool $withscores = false): RedisCluster|array|false {}

    public function vgetattr(string $key, mixed $member, bool $decode = true): RedisCluster|array|string|false {}

    public function vsetattr(string $key, mixed $member, array|string $attributes): RedisCluster|int|false {}

    public function gcra(
        string $key,
        int $maxBurst,
        int $requestsPerPeriod,
        int $period,
        int $tokens = 0,
    ): RedisCluster|array|false {}

    public function xack(string $key, string $group, array $ids): RedisCluster|int|false {}

    public function xackdel(string $key, string $group, array $ids, ?string $mode = null): RedisCluster|array|false {}

    public function xadd(
        string $key,
        string $id,
        array $values,
        int $maxlen = 0,
        bool $approx = false,
    ): RedisCluster|string|false {}

    public function xclaim(
        string $key,
        string $group,
        string $consumer,
        int $min_iddle,
        array $ids,
        array $options,
    ): RedisCluster|string|array|false {}

    public function xdel(string $key, array $ids): RedisCluster|int|false {}

    public function xdelex(string $key, array $ids, ?string $mode = null): RedisCluster|array|false {}

    public function xgroup(
        string $operation,
        ?string $key = null,
        ?string $group = null,
        ?string $id_or_consumer = null,
        bool $mkstream = false,
        int $entries_read = -2,
    ): mixed {}

    public function xautoclaim(
        string $key,
        string $group,
        string $consumer,
        int $min_idle,
        string $start,
        int $count = -1,
        bool $justid = false,
    ): RedisCluster|bool|array {}

    public function xinfo(string $operation, ?string $arg1 = null, ?string $arg2 = null, int $count = -1): mixed {}

    public function xlen(string $key): RedisCluster|int|false {}

    public function xpending(
        string $key,
        string $group,
        ?string $start = null,
        ?string $end = null,
        int $count = -1,
        ?string $consumer = null,
    ): RedisCluster|array|false {}

    public function xrange(string $key, string $start, string $end, int $count = -1): RedisCluster|bool|array {}

    public function xread(array $streams, int $count = -1, int $block = -1): RedisCluster|bool|array {}

    public function xreadgroup(
        string $group,
        string $consumer,
        array $streams,
        int $count = 1,
        int $block = 1,
    ): RedisCluster|bool|array {}

    public function xrevrange(string $key, string $start, string $end, int $count = -1): RedisCluster|bool|array {}

    public function xtrim(
        string $key,
        int $maxlen,
        bool $approx = false,
        bool $minid = false,
        int $limit = -1,
    ): RedisCluster|int|false {}

    public function zadd(
        string $key,
        array|float $score_or_options,
        mixed ...$more_scores_and_mems,
    ): RedisCluster|int|float|false {}

    public function zcard(string $key): RedisCluster|int|false {}

    public function zcount(string $key, string $start, string $end): RedisCluster|int|false {}

    public function zincrby(string $key, float $value, string $member): RedisCluster|float|false {}

    public function zinterstore(
        string $dst,
        array $keys,
        ?array $weights = null,
        ?string $aggregate = null,
    ): RedisCluster|int|false {}

    public function zintercard(array $keys, int $limit = -1): RedisCluster|int|false {}

    public function zlexcount(string $key, string $min, string $max): RedisCluster|int|false {}

    public function zpopmax(string $key, ?int $value = null): RedisCluster|bool|array {}

    public function zpopmin(string $key, ?int $value = null): RedisCluster|bool|array {}

    public function zrange(
        string $key,
        mixed $start,
        mixed $end,
        array|bool|null $options = null,
    ): RedisCluster|array|bool {}

    public function zrangestore(
        string $dstkey,
        string $srckey,
        int $start,
        int $end,
        array|bool|null $options = null,
    ): RedisCluster|int|false {}

    public function zrandmember(string $key, ?array $options = null): RedisCluster|string|array {}

    public function zrangebylex(
        string $key,
        string $min,
        string $max,
        int $offset = -1,
        int $count = -1,
    ): RedisCluster|array|false {}

    public function zrangebyscore(
        string $key,
        string $start,
        string $end,
        array $options = [],
    ): RedisCluster|array|false {}

    public function zrank(string $key, mixed $member): RedisCluster|int|false {}

    public function zrem(string $key, string $value, string ...$other_values): RedisCluster|int|false {}

    public function zremrangebylex(string $key, string $min, string $max): RedisCluster|int|false {}

    public function zremrangebyrank(string $key, string $min, string $max): RedisCluster|int|false {}

    public function zremrangebyscore(string $key, string $min, string $max): RedisCluster|int|false {}

    public function zrevrange(string $key, string $min, string $max, ?array $options = null): RedisCluster|bool|array {}

    public function zrevrangebylex(
        string $key,
        string $max,
        string $min,
        int $offset = -1,
        int $count = -1,
    ): RedisCluster|array|false {}

    public function zrevrangebyscore(
        string $key,
        string $min,
        string $max,
        ?array $options = null,
    ): RedisCluster|bool|array {}

    public function zrevrank(string $key, mixed $member): RedisCluster|int|false {}

    public function zscan(
        string $key,
        int|string|null &$iterator,
        ?string $pattern = null,
        int $count = 0,
    ): RedisCluster|bool|array {}

    public function zscore(string $key, mixed $member): RedisCluster|float|false {}

    public function zmscore(string $key, mixed $member, mixed ...$other_members): RedisCluster|array|false {}

    public function zunionstore(
        string $dst,
        array $keys,
        ?array $weights = null,
        ?string $aggregate = null,
    ): RedisCluster|int|false {}

    public function zinter(array $keys, ?array $weights = null, ?array $options = null): RedisCluster|array|false {}

    public function zdiffstore(string $dst, array $keys): RedisCluster|int|false {}

    public function zunion(array $keys, ?array $weights = null, ?array $options = null): RedisCluster|array|false {}

    public function zdiff(array $keys, ?array $options = null): RedisCluster|array|false {}

    public function digest(string $key): RedisCluster|string|false {}
}

class RedisClusterException extends RuntimeException {}
