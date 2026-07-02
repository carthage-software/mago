<?php

// Error / warning selectors

const SQLSRV_ERR_ERRORS = 0;

const SQLSRV_ERR_WARNINGS = 1;

const SQLSRV_ERR_ALL = 2;

// Logging subsystems

const SQLSRV_LOG_SYSTEM_ALL = -1;

const SQLSRV_LOG_SYSTEM_OFF = 0;

const SQLSRV_LOG_SYSTEM_INIT = 1;

const SQLSRV_LOG_SYSTEM_CONN = 2;

const SQLSRV_LOG_SYSTEM_STMT = 4;

const SQLSRV_LOG_SYSTEM_UTIL = 8;

// Logging severities

const SQLSRV_LOG_SEVERITY_ALL = -1;

const SQLSRV_LOG_SEVERITY_ERROR = 1;

const SQLSRV_LOG_SEVERITY_WARNING = 2;

const SQLSRV_LOG_SEVERITY_NOTICE = 4;

// Fetch types

const SQLSRV_FETCH_NUMERIC = 1;

const SQLSRV_FETCH_ASSOC = 2;

const SQLSRV_FETCH_BOTH = 3;

// PHP types (values 4 and 6 belong to SQLSRV_PHPTYPE_STRING() and SQLSRV_PHPTYPE_STREAM())

const SQLSRV_PHPTYPE_NULL = 1;

const SQLSRV_PHPTYPE_INT = 2;

const SQLSRV_PHPTYPE_FLOAT = 3;

const SQLSRV_PHPTYPE_DATETIME = 5;

const SQLSRV_PHPTYPE_TABLE = 7;

// Encodings

const SQLSRV_ENC_BINARY = 'binary';

const SQLSRV_ENC_CHAR = 'char';

// Nullability

const SQLSRV_NULLABLE_NO = 0;

const SQLSRV_NULLABLE_YES = 1;

const SQLSRV_NULLABLE_UNKNOWN = 2;

// SQL types

const SQLSRV_SQLTYPE_BIGINT = -5;

const SQLSRV_SQLTYPE_BIT = -7;

const SQLSRV_SQLTYPE_CHAR = 1;

const SQLSRV_SQLTYPE_DATE = 5211;

const SQLSRV_SQLTYPE_DATETIME = 25177693;

const SQLSRV_SQLTYPE_DATETIME2 = 58734173;

const SQLSRV_SQLTYPE_DATETIMEOFFSET = 58738021;

const SQLSRV_SQLTYPE_DECIMAL = 3;

const SQLSRV_SQLTYPE_FLOAT = 6;

const SQLSRV_SQLTYPE_IMAGE = -4;

const SQLSRV_SQLTYPE_INT = 4;

const SQLSRV_SQLTYPE_MONEY = 33564163;

const SQLSRV_SQLTYPE_NCHAR = -8;

const SQLSRV_SQLTYPE_NTEXT = -10;

const SQLSRV_SQLTYPE_NUMERIC = 2;

const SQLSRV_SQLTYPE_NVARCHAR = -9;

const SQLSRV_SQLTYPE_REAL = 7;

const SQLSRV_SQLTYPE_SMALLDATETIME = 8285;

const SQLSRV_SQLTYPE_SMALLINT = 5;

const SQLSRV_SQLTYPE_SMALLMONEY = 33559555;

const SQLSRV_SQLTYPE_TABLE = -153;

const SQLSRV_SQLTYPE_TEXT = -1;

const SQLSRV_SQLTYPE_TIME = 58728806;

const SQLSRV_SQLTYPE_TIMESTAMP = 4606;

const SQLSRV_SQLTYPE_TINYINT = -6;

const SQLSRV_SQLTYPE_UDT = -151;

const SQLSRV_SQLTYPE_UNIQUEIDENTIFIER = -11;

const SQLSRV_SQLTYPE_VARBINARY = -3;

const SQLSRV_SQLTYPE_VARCHAR = 12;

const SQLSRV_SQLTYPE_XML = -152;

// Parameter directions

const SQLSRV_PARAM_IN = 1;

const SQLSRV_PARAM_INOUT = 2;

const SQLSRV_PARAM_OUT = 4;

// Transaction isolation levels

const SQLSRV_TXN_READ_UNCOMMITTED = 1;

const SQLSRV_TXN_READ_COMMITTED = 2;

const SQLSRV_TXN_REPEATABLE_READ = 4;

const SQLSRV_TXN_SERIALIZABLE = 8;

const SQLSRV_TXN_SNAPSHOT = 32;

// Cursor scroll options

const SQLSRV_SCROLL_NEXT = 1;

const SQLSRV_SCROLL_FIRST = 2;

const SQLSRV_SCROLL_LAST = 3;

const SQLSRV_SCROLL_PRIOR = 4;

const SQLSRV_SCROLL_ABSOLUTE = 5;

const SQLSRV_SCROLL_RELATIVE = 6;

// Cursor types

const SQLSRV_CURSOR_FORWARD = 'forward';

const SQLSRV_CURSOR_STATIC = 'static';

const SQLSRV_CURSOR_DYNAMIC = 'dynamic';

const SQLSRV_CURSOR_KEYSET = 'keyset';

const SQLSRV_CURSOR_CLIENT_BUFFERED = 'buffered';

/**
 * @param array<string, mixed> $connectionInfo
 *
 * @return resource|false
 */
function sqlsrv_connect(string $serverName, array $connectionInfo = []): mixed {}

/**
 * @param resource $conn
 */
function sqlsrv_close($conn): bool {}

/**
 * @param resource $conn
 */
function sqlsrv_begin_transaction($conn): bool {}

/**
 * @param resource $conn
 */
function sqlsrv_commit($conn): bool {}

/**
 * @param resource $conn
 */
function sqlsrv_rollback($conn): bool {}

/**
 * @return array<int, array<string, mixed>>|null
 */
function sqlsrv_errors(int $errorsOrWarnings = SQLSRV_ERR_ALL): ?array {}

function sqlsrv_configure(string $setting, mixed $value): bool {}

function sqlsrv_get_config(string $setting): mixed {}

/**
 * @param resource             $conn
 * @param array<int, mixed>    $params
 * @param array<string, mixed> $options
 *
 * @return resource|false
 */
function sqlsrv_prepare($conn, string $sql, array $params = [], array $options = []): mixed {}

/**
 * @param resource $stmt
 */
function sqlsrv_execute($stmt): bool {}

/**
 * @param resource             $conn
 * @param array<int, mixed>    $params
 * @param array<string, mixed> $options
 *
 * @return resource|false
 */
function sqlsrv_query($conn, string $sql, array $params = [], array $options = []): mixed {}

/**
 * @param resource $stmt
 */
function sqlsrv_fetch($stmt, ?int $row = null, ?int $offset = null): ?bool {}

/**
 * @param resource $stmt
 *
 * @return array<array-key, mixed>|null|false
 */
function sqlsrv_fetch_array($stmt, ?int $fetchType = null, ?int $row = null, ?int $offset = null): array|false|null {}

/**
 * @param resource          $stmt
 * @param class-string|null      $className
 * @param array<int, mixed>|null $ctorParams
 *
 * @return object|null|false
 */
function sqlsrv_fetch_object(
    $stmt,
    ?string $className = null,
    ?array $ctorParams = null,
    ?int $row = null,
    ?int $offset = null,
): object|false|null {}

/**
 * @param resource $stmt
 */
function sqlsrv_next_result($stmt): ?bool {}

/**
 * @param resource $stmt
 */
function sqlsrv_get_field($stmt, int $fieldIndex, ?int $getAsType = null): mixed {}

/**
 * @param resource $stmt
 *
 * @return array<int, array<string, mixed>>|false
 */
function sqlsrv_field_metadata($stmt): array|false {}

/**
 * @param resource $stmt
 */
function sqlsrv_has_rows($stmt): bool {}

/**
 * @param resource $stmt
 *
 * @return int|false
 */
function sqlsrv_num_fields($stmt): int|false {}

/**
 * @param resource $stmt
 *
 * @return int|false
 */
function sqlsrv_num_rows($stmt): int|false {}

/**
 * @param resource $stmt
 *
 * @return int|false
 */
function sqlsrv_rows_affected($stmt): int|false {}

/**
 * @param resource $conn
 *
 * @return array<string, mixed>|false
 */
function sqlsrv_client_info($conn): array|false {}

/**
 * @param resource $conn
 *
 * @return array<string, mixed>
 */
function sqlsrv_server_info($conn): array {}

/**
 * @param resource $stmt
 */
function sqlsrv_cancel($stmt): bool {}

/**
 * @param resource $stmt
 */
function sqlsrv_free_stmt($stmt): bool {}

/**
 * @param resource $stmt
 */
function sqlsrv_send_stream_data($stmt): bool {}

function SQLSRV_PHPTYPE_STREAM(string $encoding): int {}

function SQLSRV_PHPTYPE_STRING(string $encoding): int {}

function SQLSRV_SQLTYPE_BINARY(int $byteCount): int {}

function SQLSRV_SQLTYPE_CHAR(int $charCount): int {}

function SQLSRV_SQLTYPE_DECIMAL(int $precision, int $scale): int {}

function SQLSRV_SQLTYPE_NCHAR(int $charCount): int {}

function SQLSRV_SQLTYPE_NUMERIC(int $precision, int $scale): int {}

/**
 * @param int|'max' $charCount
 */
function SQLSRV_SQLTYPE_NVARCHAR(int|string $charCount): int {}

/**
 * @param int|'max' $byteCount
 */
function SQLSRV_SQLTYPE_VARBINARY(int|string $byteCount): int {}

/**
 * @param int|'max' $charCount
 */
function SQLSRV_SQLTYPE_VARCHAR(int|string $charCount): int {}
