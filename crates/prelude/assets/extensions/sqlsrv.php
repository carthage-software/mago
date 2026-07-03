<?php

const SQLSRV_ERR_ERRORS = 0;

const SQLSRV_ERR_WARNINGS = 1;

const SQLSRV_ERR_ALL = 2;

const SQLSRV_LOG_SYSTEM_ALL = -1;

const SQLSRV_LOG_SYSTEM_OFF = 0;

const SQLSRV_LOG_SYSTEM_INIT = 1;

const SQLSRV_LOG_SYSTEM_CONN = 2;

const SQLSRV_LOG_SYSTEM_STMT = 4;

const SQLSRV_LOG_SYSTEM_UTIL = 8;

const SQLSRV_LOG_SEVERITY_ALL = -1;

const SQLSRV_LOG_SEVERITY_ERROR = 1;

const SQLSRV_LOG_SEVERITY_WARNING = 2;

const SQLSRV_LOG_SEVERITY_NOTICE = 4;

const SQLSRV_FETCH_NUMERIC = 1;

const SQLSRV_FETCH_ASSOC = 2;

const SQLSRV_FETCH_BOTH = 3;

const SQLSRV_PHPTYPE_NULL = 1;

const SQLSRV_PHPTYPE_INT = 2;

const SQLSRV_PHPTYPE_FLOAT = 3;

const SQLSRV_PHPTYPE_DATETIME = 5;

const SQLSRV_PHPTYPE_TABLE = 7;

const SQLSRV_ENC_BINARY = 'binary';

const SQLSRV_ENC_CHAR = 'char';

const SQLSRV_NULLABLE_NO = 0;

const SQLSRV_NULLABLE_YES = 1;

const SQLSRV_NULLABLE_UNKNOWN = 2;

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

const SQLSRV_PARAM_IN = 1;

const SQLSRV_PARAM_INOUT = 2;

const SQLSRV_PARAM_OUT = 4;

const SQLSRV_TXN_READ_UNCOMMITTED = 1;

const SQLSRV_TXN_READ_COMMITTED = 2;

const SQLSRV_TXN_REPEATABLE_READ = 4;

const SQLSRV_TXN_SERIALIZABLE = 8;

const SQLSRV_TXN_SNAPSHOT = 32;

const SQLSRV_SCROLL_NEXT = 1;

const SQLSRV_SCROLL_FIRST = 2;

const SQLSRV_SCROLL_LAST = 3;

const SQLSRV_SCROLL_PRIOR = 4;

const SQLSRV_SCROLL_ABSOLUTE = 5;

const SQLSRV_SCROLL_RELATIVE = 6;

const SQLSRV_CURSOR_FORWARD = 'forward';

const SQLSRV_CURSOR_STATIC = 'static';

const SQLSRV_CURSOR_DYNAMIC = 'dynamic';

const SQLSRV_CURSOR_KEYSET = 'keyset';

const SQLSRV_CURSOR_CLIENT_BUFFERED = 'buffered';

/**
 * @param non-empty-string     $server_name
 * @param array<string, mixed> $connection_info
 *
 * @return resource|false
 */
function sqlsrv_connect(string $server_name, array $connection_info = []): mixed {}

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
 * @return non-empty-list<array<string, mixed>>|null
 */
function sqlsrv_errors(int $errors_and_or_warnings = SQLSRV_ERR_ALL): ?array {}

function sqlsrv_configure(string $setting, mixed $value): bool {}

function sqlsrv_get_config(string $setting): mixed {}

/**
 * @param resource             $conn
 * @param array<int, mixed>    $params
 * @param array<string, mixed> $options
 *
 * @return resource|false
 */
function sqlsrv_prepare($conn, string $tsql, array $params = [], array $options = []): mixed {}

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
function sqlsrv_query($conn, string $tsql, array $params = [], array $options = []): mixed {}

/**
 * @param resource $stmt
 */
function sqlsrv_fetch($stmt, ?int $row = null, ?int $offset = null): ?bool {}

/**
 * @param resource $stmt
 *
 * @return array<array-key, mixed>|null|false
 */
function sqlsrv_fetch_array($stmt, ?int $fetch_type = null, ?int $row = null, ?int $offset = null): array|false|null {}

/**
 * @param resource               $stmt
 * @param class-string|null      $class_name
 * @param array<int, mixed>|null $ctor_params
 *
 * @return object|null|false
 */
function sqlsrv_fetch_object(
    $stmt,
    ?string $class_name = null,
    ?array $ctor_params = null,
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
function sqlsrv_get_field($stmt, int $field_index, ?int $get_as_type = null): mixed {}

/**
 * @param resource $stmt
 *
 * @return list<array<string, mixed>>|false
 */
function sqlsrv_field_metadata($stmt): array|false {}

/**
 * @param resource $stmt
 */
function sqlsrv_has_rows($stmt): bool {}

/**
 * @param resource $stmt
 *
 * @return int<0, max>|false
 */
function sqlsrv_num_fields($stmt): int|false {}

/**
 * @param resource $stmt
 *
 * @return int<0, max>|false
 */
function sqlsrv_num_rows($stmt): int|false {}

/**
 * @param resource $stmt
 *
 * @return int<-1, max>|false
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

/**
 * @param 'binary'|'char' $encoding
 */
function SQLSRV_PHPTYPE_STREAM(string $encoding): int {}

/**
 * @param 'binary'|'char' $encoding
 */
function SQLSRV_PHPTYPE_STRING(string $encoding): int {}

/**
 * @param int<1, 8000> $size
 */
function SQLSRV_SQLTYPE_BINARY(int $size): int {}

/**
 * @param int<1, 8000> $size
 */
function SQLSRV_SQLTYPE_CHAR(int $size): int {}

/**
 * @param int<1, 38> $precision
 * @param int<0, 38> $scale
 */
function SQLSRV_SQLTYPE_DECIMAL(int $precision, int $scale): int {}

/**
 * @param int<1, 4000> $size
 */
function SQLSRV_SQLTYPE_NCHAR(int $size): int {}

/**
 * @param int<1, 38> $precision
 * @param int<0, 38> $scale
 */
function SQLSRV_SQLTYPE_NUMERIC(int $precision, int $scale): int {}

/**
 * @param int<1, 4000>|'max' $size
 */
function SQLSRV_SQLTYPE_NVARCHAR(int|string $size): int {}

/**
 * @param int<1, 8000>|'max' $size
 */
function SQLSRV_SQLTYPE_VARBINARY(int|string $size): int {}

/**
 * @param int<1, 8000>|'max' $size
 */
function SQLSRV_SQLTYPE_VARCHAR(int|string $size): int {}
