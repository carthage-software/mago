<?php

class Memcached
{
    public const int OPT_SERIALIZER = -1003;

    public const int OPT_HASH = 2;
    public const int HASH_DEFAULT = 0;
    public const int HASH_MD5 = 1;
    public const int HASH_CRC = 2;
    public const int HASH_FNV1_64 = 3;
    public const int HASH_FNV1A_64 = 4;
    public const int HASH_FNV1_32 = 5;
    public const int HASH_FNV1A_32 = 6;
    public const int HASH_HSIEH = 7;
    public const int HASH_MURMUR = 8;

    public const int OPT_DISTRIBUTION = 9;
    public const int DISTRIBUTION_MODULA = 0;
    public const int DISTRIBUTION_CONSISTENT = 1;
    public const int DISTRIBUTION_VIRTUAL_BUCKET = 6;

    public const int OPT_LIBKETAMA_COMPATIBLE = 16;
    public const int OPT_LIBKETAMA_HASH = 17;
    public const int OPT_TCP_KEEPALIVE = 32;
    public const int OPT_BUFFER_WRITES = 10;
    public const int OPT_BINARY_PROTOCOL = 18;
    public const int OPT_NO_BLOCK = 0;
    public const int OPT_TCP_NODELAY = 1;
    public const int OPT_SOCKET_SEND_SIZE = 4;
    public const int OPT_SOCKET_RECV_SIZE = 5;
    public const int OPT_CONNECT_TIMEOUT = 14;
    public const int OPT_RETRY_TIMEOUT = 15;
    public const int OPT_DEAD_TIMEOUT = 36;
    public const int OPT_SEND_TIMEOUT = 19;
    public const int OPT_RECV_TIMEOUT = 20;
    public const int OPT_POLL_TIMEOUT = 8;
    public const int OPT_SERVER_FAILURE_LIMIT = 21;
    public const int OPT_SERVER_TIMEOUT_LIMIT = 37;
    public const int OPT_CACHE_LOOKUPS = 6;
    public const int OPT_AUTO_EJECT_HOSTS = 28;
    public const int OPT_NOREPLY = 26;
    public const int OPT_VERIFY_KEY = 13;
    public const int OPT_USE_UDP = 27;
    public const int OPT_NUMBER_OF_REPLICAS = 29;
    public const int OPT_RANDOMIZE_REPLICA_READS = 30;
    public const int OPT_REMOVE_FAILED_SERVERS = 35;

    /** @var bool */
    public const bool HAVE_JSON = false;
    /** @var bool */
    public const bool HAVE_IGBINARY = false;
    /** @var bool */
    public const bool HAVE_MSGPACK = false;
    /** @var bool */
    public const bool HAVE_ENCODING = false;

    /** @var bool */
    public const bool HAVE_SESSION = false;
    /** @var bool */
    public const bool HAVE_SASL = false;

    public const int OPT_COMPRESSION = -1001;
    public const int OPT_COMPRESSION_TYPE = -1004;
    public const int OPT_PREFIX_KEY = -1002;

    public const int SERIALIZER_PHP = 1;
    public const int SERIALIZER_IGBINARY = 2;
    public const int SERIALIZER_JSON = 3;
    public const int SERIALIZER_JSON_ARRAY = 4;
    public const int SERIALIZER_MSGPACK = 5;

    public const int COMPRESSION_FASTLZ = 2;
    public const int COMPRESSION_ZLIB = 1;
    public const int COMPRESSION_ZSTD = 3;

    public const int GET_PRESERVE_ORDER = 1;
    public const int GET_EXTENDED = 2;

    public const bool GET_ERROR_RETURN_VALUE = false;
    public const int RES_PAYLOAD_FAILURE = -1001;
    public const int RES_SUCCESS = 0;
    public const int RES_FAILURE = 1;
    public const int RES_HOST_LOOKUP_FAILURE = 2;
    public const int RES_UNKNOWN_READ_FAILURE = 7;
    public const int RES_PROTOCOL_ERROR = 8;
    public const int RES_CLIENT_ERROR = 9;
    public const int RES_SERVER_ERROR = 10;
    public const int RES_WRITE_FAILURE = 5;
    public const int RES_DATA_EXISTS = 12;
    public const int RES_NOTSTORED = 14;
    public const int RES_NOTFOUND = 16;
    public const int RES_PARTIAL_READ = 18;
    public const int RES_SOME_ERRORS = 19;
    public const int RES_NO_SERVERS = 20;
    public const int RES_END = 21;
    public const int RES_ERRNO = 26;
    public const int RES_BUFFERED = 32;
    public const int RES_TIMEOUT = 31;
    public const int RES_BAD_KEY_PROVIDED = 33;
    public const int RES_STORED = 15;
    public const int RES_DELETED = 22;
    public const int RES_STAT = 24;
    public const int RES_ITEM = 25;
    public const int RES_NOT_SUPPORTED = 28;
    public const int RES_FETCH_NOTFINISHED = 30;
    public const int RES_SERVER_MARKED_DEAD = 35;
    public const int RES_UNKNOWN_STAT_KEY = 36;
    public const int RES_INVALID_HOST_PROTOCOL = 34;
    public const int RES_MEMORY_ALLOCATION_FAILURE = 17;
    public const int RES_CONNECTION_SOCKET_CREATE_FAILURE = 11;
    public const int RES_E2BIG = 37;
    public const int RES_KEY_TOO_BIG = 39;
    public const int RES_SERVER_TEMPORARILY_DISABLED = 47;
    public const int RES_SERVER_MEMORY_ALLOCATION_FAILURE = 48;
    public const int RES_AUTH_PROBLEM = 40;
    public const int RES_AUTH_FAILURE = 41;
    public const int RES_AUTH_CONTINUE = 42;

    public const int ON_CONNECT = 0;
    public const int ON_ADD = 1;
    public const int ON_APPEND = 2;
    public const int ON_DECREMENT = 3;
    public const int ON_DELETE = 4;
    public const int ON_FLUSH = 5;
    public const int ON_GET = 6;
    public const int ON_INCREMENT = 7;
    public const int ON_NOOP = 8;
    public const int ON_PREPEND = 9;
    public const int ON_QUIT = 10;
    public const int ON_REPLACE = 11;
    public const int ON_SET = 12;
    public const int ON_STAT = 13;
    public const int ON_VERSION = 14;

    public const int RESPONSE_SUCCESS = 0;
    public const int RESPONSE_KEY_ENOENT = 1;
    public const int RESPONSE_KEY_EEXISTS = 2;
    public const int RESPONSE_E2BIG = 3;
    public const int RESPONSE_EINVAL = 4;
    public const int RESPONSE_NOT_STORED = 5;
    public const int RESPONSE_DELTA_BADVAL = 6;
    public const int RESPONSE_NOT_MY_VBUCKET = 7;
    public const int RESPONSE_AUTH_ERROR = 32;
    public const int RESPONSE_AUTH_CONTINUE = 33;
    public const int RESPONSE_UNKNOWN_COMMAND = 129;
    public const int RESPONSE_ENOMEM = 130;
    public const int RESPONSE_NOT_SUPPORTED = 131;
    public const int RESPONSE_EINTERNAL = 132;
    public const int RESPONSE_EBUSY = 133;
    public const int RESPONSE_ETMPFAIL = 134;

    public function __construct(
        ?string $persistent_id = null,
        ?callable $callback = null,
        ?string $connection_str = null,
    ) {}

    public function add(string $key, mixed $value, int $expiration = 0): bool {}

    public function addByKey(string $server_key, string $key, mixed $value, int $expiration = 0): bool {}

    public function addServer(string $host, int $port, int $weight = 0): bool {}

    /** @param array<array{0:string,1:int,2?:int}> $servers */
    public function addServers(array $servers): bool {}

    public function append(string $key, string $value): ?bool {}

    public function appendByKey(string $server_key, string $key, string $value): ?bool {}

    public function cas(
        #[SensitiveParameter]
        string|int|float $cas_token,
        string $key,
        mixed $value,
        int $expiration = 0,
    ): bool {}

    public function casByKey(
        #[SensitiveParameter]
        string|int|float $cas_token,
        string $server_key,
        string $key,
        mixed $value,
        int $expiration = 0,
    ): bool {}

    public function decrement(string $key, int $offset = 1, int $initial_value = 0, int $expiry = 0): int|false {}

    public function decrementByKey(
        string $server_key,
        string $key,
        int $offset = 1,
        int $initial_value = 0,
        int $expiry = 0,
    ): int|false {}

    public function delete(string $key, int $time = 0): bool {}

    public function deleteByKey(string $server_key, string $key, int $time = 0): bool {}

    /**
     * @param string[] $keys
     * @return array<string,true|int>
     */
    public function deleteMulti(array $keys, int $time = 0): array {}

    /**
     * @param string[] $keys
     * @return array<string,true|int>
     */
    public function deleteMultiByKey(string $server_key, array $keys, int $time = 0): array {}

    /** @return false|array{key:string, value: mixed, cas?: int} */
    public function fetch(): array|false {}

    /** @return false|list<array{key:string, value: mixed, cas?: int}> */
    public function fetchAll(): array|false {}

    public function flush(int $delay = 0): bool {}

    public function get(string $key, ?callable $cache_cb = null, int $get_flags = 0): mixed {}

    /** @return false|string[] */
    public function getAllKeys(): array|false {}

    public function getByKey(string $server_key, string $key, ?callable $cache_cb = null, int $get_flags = 0): mixed {}

    /** @param string[] $keys */
    public function getDelayed(array $keys, bool $with_cas = false, ?callable $value_cb = null): bool {}

    /** @param string[] $keys */
    public function getDelayedByKey(
        string $server_key,
        array $keys,
        bool $with_cas = false,
        ?callable $value_cb = null,
    ): bool {}

    /**
     * @param string[] $keys
     * @return array<string,mixed>
     */
    public function getMulti(array $keys, int $get_flags = 0): array|false {}

    /**
     * @param string[] $keys
     * @return array<string,mixed>
     */
    public function getMultiByKey(string $server_key, array $keys, int $get_flags = 0): array|false {}

    public function getOption(int $option): mixed {}

    public function getResultCode(): int {}

    public function getResultMessage(): string {}

    /** @return array<string,scalar> */
    public function getServerByKey(string $server_key): array|false {}

    /** @return list<array<string,scalar>> */
    public function getServerList(): array {}

    /** @return false|array<string,array<string,mixed>> */
    public function getStats(?string $type = null): array|false {}

    /** @return false|array<string,string> */
    public function getVersion(): array|false {}

    public function increment(string $key, int $offset = 1, int $initial_value = 0, int $expiry = 0): int|false {}

    public function incrementByKey(
        string $server_key,
        string $key,
        int $offset = 1,
        int $initial_value = 0,
        int $expiry = 0,
    ): int|false {}

    public function isPersistent(): bool {}

    public function isPristine(): bool {}

    public function prepend(string $key, string $value): ?bool {}

    public function prependByKey(string $server_key, string $key, string $value): ?bool {}

    public function quit(): bool {}

    public function replace(string $key, mixed $value, int $expiration = 0): bool {}

    public function replaceByKey(string $server_key, string $key, mixed $value, int $expiration = 0): bool {}

    public function resetServerList(): bool {}

    public function set(string $key, mixed $value, int $expiration = 0): bool {}

    public function setByKey(string $server_key, string $key, mixed $value, int $expiration = 0): bool {}

    public function setEncodingKey(string $key): bool {}

    /** @param array<string,mixed> $items */
    public function setMulti(array $items, int $expiration = 0): bool {}

    /** @param array<string,mixed> $items */
    public function setMultiByKey(string $server_key, array $items, int $expiration = 0): bool {}

    public function setOption(int $option, mixed $value): bool {}

    /** @param array<int,mixed> $options */
    public function setOptions(array $options): bool {}

    public function setSaslAuthData(string $username, #[SensitiveParameter] string $password): bool {}

    public function touch(string $key, int $expiration = 0): bool {}

    public function touchByKey(string $server_key, string $key, int $expiration = 0): bool {}
}

class MemcachedException extends Exception
{
    function __construct(string $errmsg = '', int $errcode = 0) {}
}
