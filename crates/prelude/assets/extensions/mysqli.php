<?php

/**
 * @var int
 */
const MYSQLI_READ_DEFAULT_GROUP = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_READ_DEFAULT_FILE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_OPT_CONNECT_TIMEOUT = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_OPT_LOCAL_INFILE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_OPT_LOAD_DATA_LOCAL_DIR = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_INIT_COMMAND = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_OPT_READ_TIMEOUT = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_OPT_NET_CMD_BUFFER_SIZE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_OPT_NET_READ_BUFFER_SIZE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_OPT_INT_AND_FLOAT_NATIVE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_OPT_SSL_VERIFY_SERVER_CERT = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_SERVER_PUBLIC_KEY = UNKNOWN;

/* mysqli_real_connect flags */
/**
 * @var int
 */
const MYSQLI_CLIENT_SSL = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_CLIENT_COMPRESS = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_CLIENT_INTERACTIVE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_CLIENT_IGNORE_SPACE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_CLIENT_NO_SCHEMA = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_CLIENT_FOUND_ROWS = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_CLIENT_SSL_VERIFY_SERVER_CERT = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_CLIENT_SSL_DONT_VERIFY_SERVER_CERT = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_CLIENT_CAN_HANDLE_EXPIRED_PASSWORDS = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_OPT_CAN_HANDLE_EXPIRED_PASSWORDS = UNKNOWN;

/* for mysqli_query */
/**
 * @var int
 */
const MYSQLI_STORE_RESULT = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_USE_RESULT = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_ASYNC = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.4', message: 'as the mysqli_store_result() parameter is unused since 8.1')]
const MYSQLI_STORE_RESULT_COPY_DATA = UNKNOWN;

/* for mysqli_fetch_assoc */
/**
 * @var int
 */
const MYSQLI_ASSOC = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_NUM = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_BOTH = UNKNOWN;

/* for mysqli_stmt_set_attr */
/**
 * @var int
 */
const MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_STMT_ATTR_CURSOR_TYPE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_CURSOR_TYPE_NO_CURSOR = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_CURSOR_TYPE_READ_ONLY = UNKNOWN;

/* column information */
/**
 * @var int
 */
const MYSQLI_NOT_NULL_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_PRI_KEY_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_UNIQUE_KEY_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_MULTIPLE_KEY_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_BLOB_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_UNSIGNED_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_ZEROFILL_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_AUTO_INCREMENT_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TIMESTAMP_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_SET_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_NUM_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_PART_KEY_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_GROUP_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_ENUM_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_BINARY_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_NO_DEFAULT_VALUE_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_ON_UPDATE_NOW_FLAG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_DECIMAL = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_TINY = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_SHORT = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_LONG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_FLOAT = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_DOUBLE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_NULL = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_TIMESTAMP = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_LONGLONG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_INT24 = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_DATE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_TIME = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_DATETIME = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_YEAR = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_NEWDATE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_ENUM = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_SET = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_TINY_BLOB = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_MEDIUM_BLOB = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_LONG_BLOB = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_BLOB = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_VAR_STRING = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_STRING = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_CHAR = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_GEOMETRY = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_VECTOR = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_JSON = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_NEWDECIMAL = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TYPE_BIT = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_SET_CHARSET_NAME = UNKNOWN;

/* bind support */
/**
 * @var int
 */
#[Deprecated(since: '8.1', message: 'as it was unused')]
const MYSQLI_NO_DATA = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.1', message: 'as it was unused')]
const MYSQLI_DATA_TRUNCATED = UNKNOWN;

/* reporting */
/**
 * @var int
 */
const MYSQLI_REPORT_INDEX = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_REPORT_ERROR = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_REPORT_STRICT = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_REPORT_ALL = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_REPORT_OFF = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_DEBUG_TRACE_ENABLED = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.1', message: 'as it was unused')]
const MYSQLI_SERVER_QUERY_NO_GOOD_INDEX_USED = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.1', message: 'as it was unused')]
const MYSQLI_SERVER_QUERY_NO_INDEX_USED = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.1', message: 'as it was unused')]
const MYSQLI_SERVER_QUERY_WAS_SLOW = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.1', message: 'as it was unused')]
const MYSQLI_SERVER_PS_OUT_PARAMS = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.4', message: 'as mysqli_refresh() is deprecated')]
const MYSQLI_REFRESH_GRANT = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.4', message: 'as mysqli_refresh() is deprecated')]
const MYSQLI_REFRESH_LOG = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.4', message: 'as mysqli_refresh() is deprecated')]
const MYSQLI_REFRESH_TABLES = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.4', message: 'as mysqli_refresh() is deprecated')]
const MYSQLI_REFRESH_HOSTS = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.4', message: 'as mysqli_refresh() is deprecated')]
const MYSQLI_REFRESH_STATUS = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.4', message: 'as mysqli_refresh() is deprecated')]
const MYSQLI_REFRESH_THREADS = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.4', message: 'as mysqli_refresh() is deprecated')]
const MYSQLI_REFRESH_REPLICA = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.4', message: 'as mysqli_refresh() is deprecated')]
const MYSQLI_REFRESH_SLAVE = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.4', message: 'as mysqli_refresh() is deprecated')]
const MYSQLI_REFRESH_MASTER = UNKNOWN;

/**
 * @var int
 */
#[Deprecated(since: '8.4', message: 'as mysqli_refresh() is deprecated')]
const MYSQLI_REFRESH_BACKUP_LOG = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TRANS_START_WITH_CONSISTENT_SNAPSHOT = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TRANS_START_READ_WRITE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TRANS_START_READ_ONLY = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TRANS_COR_AND_CHAIN = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TRANS_COR_AND_NO_CHAIN = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TRANS_COR_RELEASE = UNKNOWN;

/**
 * @var int
 */
const MYSQLI_TRANS_COR_NO_RELEASE = UNKNOWN;

/**
 * @var bool
 */
#[Deprecated(since: '8.2', message: 'as it is always false')]
const MYSQLI_IS_MARIADB = false;

final class mysqli_driver
{
    /** @readonly */
    public string $client_info;

    /** @readonly */
    public int $client_version;

    /** @readonly */
    public int $driver_version;

    public int $report_mode = 0;
}

class mysqli
{
    /**
     * @readonly
     */
    public int|string $affected_rows;

    /**
     * @readonly
     */
    public string $client_info;

    /**
     * @readonly
     */
    public int $client_version;

    /**
     * @readonly
     */
    public int $connect_errno;

    /**
     * @readonly
     */
    public ?string $connect_error;

    /**
     * @readonly
     */
    public int $errno;

    /**
     * @readonly
     */
    public string $error;

    /**
     * @readonly
     */
    public array $error_list;

    /**
     * @readonly
     */
    public int $field_count;

    /**
     * @readonly
     */
    public string $host_info;

    /**
     * @readonly
     */
    public ?string $info;

    /**
     * @readonly
     */
    public int|string $insert_id;

    /**
     * @readonly
     */
    public string $server_info;

    /**
     * @readonly
     */
    public int $server_version;

    /**
     * @readonly
     */
    public string $sqlstate;

    /**
     * @readonly
     */
    public int $protocol_version;

    /**
     * @readonly
     */
    public int $thread_id;

    /**
     * @readonly
     */
    public int $warning_count;

    public function __construct(
        ?string $hostname = null,
        ?string $username = null,
        #[\SensitiveParameter] ?string $password = null,
        ?string $database = null,
        ?int $port = null,
        ?string $socket = null,
    ) {}

    public function autocommit(bool $enable): bool {}

    public function begin_transaction(int $flags = 0, ?string $name = null): bool {}

    public function change_user(string $username, #[\SensitiveParameter] string $password, ?string $database): bool {}

    public function character_set_name(): string {}

    public function close(): true {}

    public function commit(int $flags = 0, ?string $name = null): bool {}

    public function connect(
        ?string $hostname = null,
        ?string $username = null,
        #[\SensitiveParameter] ?string $password = null,
        ?string $database = null,
        ?int $port = null,
        ?string $socket = null,
    ): bool {}

    public function dump_debug_info(): bool {}

    public function debug(string $options): true {}

    public function get_charset(): ?object {}

    public function execute_query(string $query, ?array $params = null): mysqli_result|bool {}

    #[Deprecated(since: '8.1', message: 'use mysqli_get_client_info() instead')]
    public function get_client_info(): string {}

    /**
     * @return array<string, mixed>
     */
    public function get_connection_stats(): array {}

    public function get_server_info(): string {}

    public function get_warnings(): mysqli_warning|false {}

    /**
     * @return bool|null
     * */
    #[Deprecated(since: '8.1', message: 'replace calls to parent::init() with parent::__construct()')]
    public function init() {}

    #[Deprecated(since: '8.4', message: 'use KILL CONNECTION/QUERY SQL statement instead')]
    public function kill(int $process_id): bool {}

    public function multi_query(string $query): bool {}

    public function more_results(): bool {}

    public function next_result(): bool {}

    #[Deprecated(
        since: '8.4',
        message: 'because the reconnect feature has been removed in PHP 8.2 and this method is now redundant',
    )]
    public function ping(): bool {}

    public static function poll(
        ?array &$read,
        ?array &$error,
        array &$reject,
        int $seconds,
        int $microseconds = 0,
    ): int|false {}

    public function prepare(string $query): mysqli_stmt|false {}

    public function query(string $query, int $result_mode = MYSQLI_STORE_RESULT): mysqli_result|bool {}

    public function real_connect(
        ?string $hostname = null,
        ?string $username = null,
        #[\SensitiveParameter] ?string $password = null,
        ?string $database = null,
        ?int $port = null,
        ?string $socket = null,
        int $flags = 0,
    ): bool {}

    public function real_escape_string(string $string): string {}

    public function reap_async_query(): mysqli_result|bool {}

    public function escape_string(string $string): string {}

    public function real_query(string $query): bool {}

    public function release_savepoint(string $name): bool {}

    public function rollback(int $flags = 0, ?string $name = null): bool {}

    public function savepoint(string $name): bool {}

    public function select_db(string $database): bool {}

    public function set_charset(string $charset): bool {}

    /**
     * @param string|int $value
     */
    public function options(int $option, $value): bool {}

    /**
     * @param string|int $value
     */
    public function set_opt(int $option, $value): bool {}

    public function ssl_set(
        ?string $key,
        ?string $certificate,
        ?string $ca_certificate,
        ?string $ca_path,
        ?string $cipher_algos,
    ): true {}

    public function stat(): string|false {}

    public function stmt_init(): mysqli_stmt|false {}

    public function store_result(int $mode = 0): mysqli_result|false {}

    public function thread_safe(): bool {}

    public function use_result(): mysqli_result|false {}

    #[Deprecated(since: '8.4', message: 'use FLUSH SQL statement instead')]
    public function refresh(int $flags): bool {}
}

class mysqli_result implements IteratorAggregate
{
    /**
     * @readonly
     */
    public int $current_field;

    /**
     * @readonly
     */
    public int $field_count;

    /**
     * @readonly
     */
    public ?array $lengths;

    /**
     * @readonly
     */
    public int|string $num_rows;

    public int $type;

    public function __construct(mysqli $mysql, int $result_mode = MYSQLI_STORE_RESULT) {}

    public function close(): void {}

    public function free(): void {}

    public function data_seek(int $offset): bool {}

    public function fetch_field(): object|false {}

    /**
     * @return array<int, object>
     */
    public function fetch_fields(): array {}

    public function fetch_field_direct(int $index): object|false {}

    /**
     * @return array<int|string, mixed>
     */
    public function fetch_all(int $mode = MYSQLI_NUM): array {}

    /**
     * @return array<int|string, mixed>|null|false
     */
    public function fetch_array(int $mode = MYSQLI_BOTH): array|null|false {}

    /**
     * @return array<int|string, mixed>|null|false
     */
    public function fetch_assoc(): array|null|false {}

    public function fetch_object(string $class = 'stdClass', array $constructor_args = []): object|null|false {}

    /**
     * @return array<int, mixed>|null|false
     */
    public function fetch_row(): array|null|false {}

    public function fetch_column(int $column = 0): null|int|float|string|false {}

    public function field_seek(int $index): true {}

    public function free_result(): void {}

    public function getIterator(): Iterator {}
}

class mysqli_stmt
{
    /**
     * @readonly
     */
    public int|string $affected_rows;

    /**
     * @readonly
     */
    public int|string $insert_id;

    /**
     * @readonly
     */
    public int|string $num_rows;

    /**
     * @readonly
     */
    public int $param_count;

    /**
     * @readonly
     */
    public int $field_count;

    /**
     * @readonly
     */
    public int $errno;

    /**
     * @readonly
     */
    public string $error;

    /**
     * @readonly
     */
    public array $error_list;

    /**
     * @readonly
     */
    public string $sqlstate;

    public int $id;

    public function __construct(mysqli $mysql, ?string $query = null) {}

    public function attr_get(int $attribute): int {}

    public function attr_set(int $attribute, int $value): bool {}

    public function bind_param(string $types, mixed &...$vars): bool {}

    public function bind_result(mixed &...$vars): bool {}

    public function close(): true {}

    public function data_seek(int $offset): void {}

    public function execute(?array $params = null): bool {}

    public function fetch(): ?bool {}

    public function get_warnings(): mysqli_warning|false {}

    public function result_metadata(): mysqli_result|false {}

    public function more_results(): bool {}

    public function next_result(): bool {}

    public function num_rows(): int|string {}

    public function send_long_data(int $param_num, string $data): bool {}

    public function free_result(): void {}

    public function reset(): bool {}

    public function prepare(string $query): bool {}

    public function store_result(): bool {}

    public function get_result(): mysqli_result|false {}
}

final class mysqli_warning
{
    public string $message;

    public string $sqlstate;

    public int $errno;

    private function __construct() {}

    public function next(): bool {}
}

final class mysqli_sql_exception extends RuntimeException
{
    protected string $sqlstate = '00000';

    public function getSqlState(): string {}
}

function mysqli_affected_rows(mysqli $mysql): int|string {}

function mysqli_autocommit(mysqli $mysql, bool $enable): bool {}

function mysqli_begin_transaction(mysqli $mysql, int $flags = 0, ?string $name = null): bool {}

function mysqli_change_user(
    mysqli $mysql,
    string $username,
    #[\SensitiveParameter] string $password,
    ?string $database,
): bool {}

function mysqli_character_set_name(mysqli $mysql): string {}

function mysqli_close(mysqli $mysql): true {}

function mysqli_commit(mysqli $mysql, int $flags = 0, ?string $name = null): bool {}

/**
 */
function mysqli_connect(
    ?string $hostname = null,
    ?string $username = null,
    #[\SensitiveParameter] ?string $password = null,
    ?string $database = null,
    ?int $port = null,
    ?string $socket = null,
): mysqli|false {}

function mysqli_connect_errno(): int {}

function mysqli_connect_error(): ?string {}

function mysqli_data_seek(mysqli_result $result, int $offset): bool {}

function mysqli_dump_debug_info(mysqli $mysql): bool {}

function mysqli_debug(string $options): true {}

function mysqli_errno(mysqli $mysql): int {}

function mysqli_error(mysqli $mysql): string {}

/**
 * @return array<int, array>
 */
function mysqli_error_list(mysqli $mysql): array {}

function mysqli_stmt_execute(mysqli_stmt $statement, ?array $params = null): bool {}

#[Deprecated(since: '8.5', message: 'use mysqli_stmt_execute() instead')]
function mysqli_execute(mysqli_stmt $statement, ?array $params = null): bool {}

function mysqli_execute_query(mysqli $mysql, string $query, ?array $params = null): mysqli_result|bool {}

function mysqli_fetch_field(mysqli_result $result): object|false {}

/**
 * @return array<int, object>
 */
function mysqli_fetch_fields(mysqli_result $result): array {}

function mysqli_fetch_field_direct(mysqli_result $result, int $index): object|false {}

/**
 * @return array<int, int>|false
 */
function mysqli_fetch_lengths(mysqli_result $result): array|false {}

/**
 * @return array<int|string, mixed>
 */
function mysqli_fetch_all(mysqli_result $result, int $mode = MYSQLI_NUM): array {}

/**
 * @return array<int|string, mixed>|null|false
 */
function mysqli_fetch_array(mysqli_result $result, int $mode = MYSQLI_BOTH): array|null|false {}

/**
 * @return array<int|string, mixed>|null|false
 */
function mysqli_fetch_assoc(mysqli_result $result): array|null|false {}

function mysqli_fetch_object(
    mysqli_result $result,
    string $class = 'stdClass',
    array $constructor_args = [],
): object|null|false {}

/**
 * @return array<int, mixed>|null|false
 */
function mysqli_fetch_row(mysqli_result $result): array|null|false {}

function mysqli_fetch_column(mysqli_result $result, int $column = 0): null|int|float|string|false {}

function mysqli_field_count(mysqli $mysql): int {}

function mysqli_field_seek(mysqli_result $result, int $index): true {}

function mysqli_field_tell(mysqli_result $result): int {}

function mysqli_free_result(mysqli_result $result): void {}

/**
 * @return array<string, mixed>
 */
function mysqli_get_connection_stats(mysqli $mysql): array {}

/**
 * @return array<string, string>
 */
function mysqli_get_client_stats(): array {}

function mysqli_get_charset(mysqli $mysql): ?object {}

function mysqli_get_client_info(?mysqli $mysql = null): string {}

function mysqli_get_client_version(): int {}

/**
 * @return array<string, int>
 */
function mysqli_get_links_stats(): array {}

function mysqli_get_host_info(mysqli $mysql): string {}

function mysqli_get_proto_info(mysqli $mysql): int {}

function mysqli_get_server_info(mysqli $mysql): string {}

function mysqli_get_server_version(mysqli $mysql): int {}

function mysqli_get_warnings(mysqli $mysql): mysqli_warning|false {}

function mysqli_init(): mysqli|false {}

function mysqli_info(mysqli $mysql): ?string {}

function mysqli_insert_id(mysqli $mysql): int|string {}

#[Deprecated(since: '8.4', message: 'use KILL CONNECTION/QUERY SQL statement instead')]
function mysqli_kill(mysqli $mysql, int $process_id): bool {}

function mysqli_more_results(mysqli $mysql): bool {}

function mysqli_multi_query(mysqli $mysql, string $query): bool {}

function mysqli_next_result(mysqli $mysql): bool {}

function mysqli_num_fields(mysqli_result $result): int {}

function mysqli_num_rows(mysqli_result $result): int|string {}

/** @param string|int $value */
function mysqli_options(mysqli $mysql, int $option, $value): bool {}

/**
 * @param string|int $value
 */
function mysqli_set_opt(mysqli $mysql, int $option, $value): bool {}

#[Deprecated(
    since: '8.4',
    message: 'because the reconnect feature has been removed in PHP 8.2 and this function is now redundant',
)]
function mysqli_ping(mysqli $mysql): bool {}

function mysqli_poll(?array &$read, ?array &$error, array &$reject, int $seconds, int $microseconds = 0): int|false {}

function mysqli_prepare(mysqli $mysql, string $query): mysqli_stmt|false {}

function mysqli_report(int $flags): true {}

function mysqli_query(mysqli $mysql, string $query, int $result_mode = MYSQLI_STORE_RESULT): mysqli_result|bool {}

function mysqli_real_connect(
    mysqli $mysql,
    ?string $hostname = null,
    ?string $username = null,
    #[\SensitiveParameter] ?string $password = null,
    ?string $database = null,
    ?int $port = null,
    ?string $socket = null,
    int $flags = 0,
): bool {}

function mysqli_real_escape_string(mysqli $mysql, string $string): string {}

function mysqli_escape_string(mysqli $mysql, string $string): string {}

function mysqli_real_query(mysqli $mysql, string $query): bool {}

function mysqli_reap_async_query(mysqli $mysql): mysqli_result|bool {}

function mysqli_release_savepoint(mysqli $mysql, string $name): bool {}

function mysqli_rollback(mysqli $mysql, int $flags = 0, ?string $name = null): bool {}

function mysqli_savepoint(mysqli $mysql, string $name): bool {}

function mysqli_select_db(mysqli $mysql, string $database): bool {}

function mysqli_set_charset(mysqli $mysql, string $charset): bool {}

function mysqli_stmt_affected_rows(mysqli_stmt $statement): int|string {}

function mysqli_stmt_attr_get(mysqli_stmt $statement, int $attribute): int {}

function mysqli_stmt_attr_set(mysqli_stmt $statement, int $attribute, int $value): bool {}

function mysqli_stmt_bind_param(mysqli_stmt $statement, string $types, mixed &...$vars): bool {}

function mysqli_stmt_bind_result(mysqli_stmt $statement, mixed &...$vars): bool {}

function mysqli_stmt_close(mysqli_stmt $statement): true {}

function mysqli_stmt_data_seek(mysqli_stmt $statement, int $offset): void {}

function mysqli_stmt_errno(mysqli_stmt $statement): int {}

function mysqli_stmt_error(mysqli_stmt $statement): string {}

/**
 * @return array<int, array>
 */
function mysqli_stmt_error_list(mysqli_stmt $statement): array {}

function mysqli_stmt_fetch(mysqli_stmt $statement): ?bool {}

function mysqli_stmt_field_count(mysqli_stmt $statement): int {}

function mysqli_stmt_free_result(mysqli_stmt $statement): void {}

function mysqli_stmt_get_result(mysqli_stmt $statement): mysqli_result|false {}

function mysqli_stmt_get_warnings(mysqli_stmt $statement): mysqli_warning|false {}

function mysqli_stmt_init(mysqli $mysql): mysqli_stmt|false {}

function mysqli_stmt_insert_id(mysqli_stmt $statement): int|string {}

function mysqli_stmt_more_results(mysqli_stmt $statement): bool {}

function mysqli_stmt_next_result(mysqli_stmt $statement): bool {}

function mysqli_stmt_num_rows(mysqli_stmt $statement): int|string {}

function mysqli_stmt_param_count(mysqli_stmt $statement): int {}

function mysqli_stmt_prepare(mysqli_stmt $statement, string $query): bool {}

function mysqli_stmt_reset(mysqli_stmt $statement): bool {}

function mysqli_stmt_result_metadata(mysqli_stmt $statement): mysqli_result|false {}

function mysqli_stmt_send_long_data(mysqli_stmt $statement, int $param_num, string $data): bool {}

function mysqli_stmt_store_result(mysqli_stmt $statement): bool {}

function mysqli_stmt_sqlstate(mysqli_stmt $statement): string {}

function mysqli_sqlstate(mysqli $mysql): string {}

function mysqli_ssl_set(
    mysqli $mysql,
    ?string $key,
    ?string $certificate,
    ?string $ca_certificate,
    ?string $ca_path,
    ?string $cipher_algos,
): true {}

function mysqli_stat(mysqli $mysql): string|false {}

function mysqli_store_result(mysqli $mysql, int $mode = 0): mysqli_result|false {}

function mysqli_thread_id(mysqli $mysql): int {}

function mysqli_thread_safe(): bool {}

function mysqli_use_result(mysqli $mysql): mysqli_result|false {}

function mysqli_warning_count(mysqli $mysql): int {}

#[Deprecated(since: '8.4', message: 'use FLUSH SQL statement instead')]
function mysqli_refresh(mysqli $mysql, int $flags): bool {}
