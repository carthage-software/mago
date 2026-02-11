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
