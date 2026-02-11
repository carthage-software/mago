<?php

namespace {
    /**
     * @var string
     */
    const PGSQL_LIBPQ_VERSION = UNKNOWN;

    /**
     * @var string
     */
    #[Deprecated(since: '8.0', message: 'as it is the same as PGSQL_LIBPQ_VERSION')]
    const PGSQL_LIBPQ_VERSION_STR = UNKNOWN;

    /* For connection option */

    /**
     * @var int
     */
    const PGSQL_CONNECT_FORCE_NEW = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_CONNECT_ASYNC = UNKNOWN;

    /* For pg_fetch_array() */

    /**
     * @var int
     */
    const PGSQL_ASSOC = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_NUM = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_BOTH = UNKNOWN;

    /* For pg_last_notice() */

    /**
     * @var int
     */
    const PGSQL_NOTICE_LAST = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_NOTICE_ALL = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_NOTICE_CLEAR = UNKNOWN;

    /* For pg_connection_status() */

    /**
     * @var int
     */
    const PGSQL_CONNECTION_BAD = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_CONNECTION_OK = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_CONNECTION_STARTED = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_CONNECTION_MADE = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_CONNECTION_AWAITING_RESPONSE = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_CONNECTION_AUTH_OK = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_CONNECTION_SSL_STARTUP = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_CONNECTION_SETENV = UNKNOWN;

    /* For pg_connect_poll() */
    /**
     * @var int
     */
    const PGSQL_POLLING_FAILED = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_POLLING_READING = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_POLLING_WRITING = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_POLLING_OK = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_POLLING_ACTIVE = UNKNOWN;

    /* For pg_transaction_status() */

    /**
     * @var int
     */
    const PGSQL_TRANSACTION_IDLE = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_TRANSACTION_ACTIVE = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_TRANSACTION_INTRANS = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_TRANSACTION_INERROR = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_TRANSACTION_UNKNOWN = UNKNOWN;

    /* For pg_set_error_verbosity() */

    /**
     * @var int
     */
    const PGSQL_ERRORS_TERSE = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_ERRORS_DEFAULT = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_ERRORS_VERBOSE = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_ERRORS_SQLSTATE = UNKNOWN;

    // else
    /**
     * @var int
     */
    const PGSQL_ERRORS_SQLSTATE = UNKNOWN;

    /* For lo_seek() */

    /**
     * @var int
     */
    const PGSQL_SEEK_SET = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_SEEK_CUR = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_SEEK_END = UNKNOWN;

    /* For pg_result_status() return value type */

    /**
     * @var int
     */
    const PGSQL_STATUS_LONG = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_STATUS_STRING = UNKNOWN;

    /* For pg_result_status() return value */

    /**
     * @var int
     */
    const PGSQL_EMPTY_QUERY = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_COMMAND_OK = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_TUPLES_OK = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_TUPLES_CHUNK = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_COPY_OUT = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_COPY_IN = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_BAD_RESPONSE = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_NONFATAL_ERROR = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_FATAL_ERROR = UNKNOWN;

    /* For pg_result_error_field() field codes */

    /**
     * @var int
     */
    const PGSQL_DIAG_SEVERITY = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_SQLSTATE = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_MESSAGE_PRIMARY = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_MESSAGE_DETAIL = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_MESSAGE_HINT = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_STATEMENT_POSITION = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_INTERNAL_POSITION = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_INTERNAL_QUERY = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_CONTEXT = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_SOURCE_FILE = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_SOURCE_LINE = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_SOURCE_FUNCTION = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_SCHEMA_NAME = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_TABLE_NAME = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_COLUMN_NAME = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_DATATYPE_NAME = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_CONSTRAINT_NAME = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DIAG_SEVERITY_NONLOCALIZED = UNKNOWN;

    /* pg_convert options */

    /**
     * @var int
     */
    const PGSQL_CONV_IGNORE_DEFAULT = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_CONV_FORCE_NULL = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_CONV_IGNORE_NOT_NULL = UNKNOWN;

    /* pg_insert/update/delete/select options */

    /**
     * @var int
     */
    const PGSQL_DML_ESCAPE = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DML_NO_CONV = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DML_EXEC = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DML_ASYNC = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_DML_STRING = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_TRACE_SUPPRESS_TIMESTAMPS = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_TRACE_REGRESS_MODE = UNKNOWN;

    /* For pg_set_error_context_visibility() */

    /**
     * @var int
     */
    const PGSQL_SHOW_CONTEXT_NEVER = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_SHOW_CONTEXT_ERRORS = UNKNOWN;

    /**
     * @var int
     */
    const PGSQL_SHOW_CONTEXT_ALWAYS = UNKNOWN;

    function pg_connect(string $connection_string, int $flags = 0): PgSql\Connection|false {}

    function pg_pconnect(string $connection_string, int $flags = 0): PgSql\Connection|false {}

    function pg_connect_poll(PgSql\Connection $connection): int {}

    function pg_close(?PgSql\Connection $connection = null): true {}

    function pg_dbname(?PgSql\Connection $connection = null): string {}

    function pg_last_error(?PgSql\Connection $connection = null): string {}

    #[Deprecated(since: '8.0', message: 'use pg_last_error() instead')]
    function pg_errormessage(?PgSql\Connection $connection = null): string {}

    function pg_options(?PgSql\Connection $connection = null): string {}

    function pg_port(?PgSql\Connection $connection = null): string {}

    function pg_tty(?PgSql\Connection $connection = null): string {}

    function pg_host(?PgSql\Connection $connection = null): string {}

    /**
     * @return array<string, int|string|null>
     */
    function pg_version(?PgSql\Connection $connection = null): array {}

    /**
     * @return array<string, string|null>
     */
    function pg_jit(?PgSql\Connection $connection = null): array {}

    function pg_service(?PgSql\Connection $connection = null): string {}

    /**
     * @param PgSql\Connection|string $connection
     */
    function pg_parameter_status($connection, string $name = UNKNOWN): string|false {}

    function pg_ping(?PgSql\Connection $connection = null): bool {}

    /**
     * @param PgSql\Connection|string $connection
     */
    function pg_query($connection, string $query = UNKNOWN): PgSql\Result|false {}

    /**
     * @param PgSql\Connection|string $connection
     */
    function pg_exec($connection, string $query = UNKNOWN): PgSql\Result|false {}

    /**
     * @param PgSql\Connection|string $connection
     * @param string|array $query
     */
    function pg_query_params($connection, $query, array $params = UNKNOWN): PgSql\Result|false {}

    /**
     * @param PgSql\Connection|string $connection
     */
    function pg_prepare($connection, string $statement_name, string $query = UNKNOWN): PgSql\Result|false {}

    /**
     * @param PgSql\Connection|string $connection
     * @param string|array $statement_name
     */
    function pg_execute($connection, $statement_name, array $params = UNKNOWN): PgSql\Result|false {}

    function pg_num_rows(PgSql\Result $result): int {}

    #[Deprecated(since: '8.0', message: 'use pg_num_rows() instead')]
    function pg_numrows(PgSql\Result $result): int {}

    function pg_num_fields(PgSql\Result $result): int {}

    #[Deprecated(since: '8.0', message: 'use pg_num_fields() instead')]
    function pg_numfields(PgSql\Result $result): int {}

    function pg_affected_rows(PgSql\Result $result): int {}

    #[Deprecated(since: '8.0', message: 'use pg_affected_rows() instead')]
    function pg_cmdtuples(PgSql\Result $result): int {}

    function pg_last_notice(PgSql\Connection $connection, int $mode = PGSQL_NOTICE_LAST): array|string|bool {}

    function pg_field_table(PgSql\Result $result, int $field, bool $oid_only = false): string|int|false {}

    function pg_field_name(PgSql\Result $result, int $field): string {}

    #[Deprecated(since: '8.0', message: 'use pg_field_name() instead')]
    function pg_fieldname(PgSql\Result $result, int $field): string {}

    function pg_field_size(PgSql\Result $result, int $field): int {}

    #[Deprecated(since: '8.0', message: 'use pg_field_size() instead')]
    function pg_fieldsize(PgSql\Result $result, int $field): int {}

    function pg_field_type(PgSql\Result $result, int $field): string {}

    #[Deprecated(since: '8.0', message: 'use pg_field_type() instead')]
    function pg_fieldtype(PgSql\Result $result, int $field): string {}

    function pg_field_type_oid(PgSql\Result $result, int $field): string|int {}

    function pg_field_num(PgSql\Result $result, string $field): int {}

    #[Deprecated(since: '8.0', message: 'use pg_field_num() instead')]
    function pg_fieldnum(PgSql\Result $result, string $field): int {}

    /**
     * @param string|int|null $row
     */
    function pg_fetch_result(PgSql\Result $result, $row, string|int $field = UNKNOWN): string|false|null {}

    /**
     * @param string|int $row
     */
    #[Deprecated(since: '8.0', message: 'use pg_fetch_result() instead')]
    function pg_result(PgSql\Result $result, $row, string|int $field = UNKNOWN): string|false|null {}

    /**
     * @return array<int|string, string|null>|false
     */
    function pg_fetch_row(PgSql\Result $result, ?int $row = null, int $mode = PGSQL_NUM): array|false {}

    /**
     * @return array<int|string, string|null>|false
     */
    function pg_fetch_assoc(PgSql\Result $result, ?int $row = null): array|false {}

    /**
     * @return array<int|string, string|null>|false
     */
    function pg_fetch_array(PgSql\Result $result, ?int $row = null, int $mode = PGSQL_BOTH): array|false {}

    function pg_fetch_object(
        PgSql\Result $result,
        ?int $row = null,
        string $class = 'stdClass',
        array $constructor_args = [],
    ): object|false {}

    /**
     * @return array<int, array>
     */
    function pg_fetch_all(PgSql\Result $result, int $mode = PGSQL_ASSOC): array {}

    /**
     * @return array<int, string|null>
     */
    function pg_fetch_all_columns(PgSql\Result $result, int $field = 0): array {}

    function pg_result_seek(PgSql\Result $result, int $row): bool {}

    /** @param string|int|null $row */
    function pg_field_prtlen(PgSql\Result $result, $row, string|int $field = UNKNOWN): int|false {}

    /**
     * @param string|int $row
     */
    #[Deprecated(since: '8.0', message: 'use pg_field_prtlen() instead')]
    function pg_fieldprtlen(PgSql\Result $result, $row, string|int $field = UNKNOWN): int|false {}

    /** @param string|int|null $row */
    function pg_field_is_null(PgSql\Result $result, $row, string|int $field = UNKNOWN): int|false {}

    /**
     * @param string|int $row
     */
    #[Deprecated(since: '8.0', message: 'use pg_field_is_null() instead')]
    function pg_fieldisnull(PgSql\Result $result, $row, string|int $field = UNKNOWN): int|false {}

    function pg_free_result(PgSql\Result $result): bool {}

    #[Deprecated(since: '8.0', message: 'use pg_free_result() instead')]
    function pg_freeresult(PgSql\Result $result): bool {}

    function pg_last_oid(PgSql\Result $result): string|int|false {}

    #[Deprecated(since: '8.0', message: 'use pg_last_oid() instead')]
    function pg_getlastoid(PgSql\Result $result): string|int|false {}

    function pg_trace(
        string $filename,
        string $mode = 'w',
        ?PgSql\Connection $connection = null,
        int $trace_mode = 0,
    ): bool {}

    function pg_untrace(?PgSql\Connection $connection = null): true {}

    /**
     * @param PgSql\Connection $connection
     * @param string|int $oid
     */
    function pg_lo_create($connection = UNKNOWN, $oid = UNKNOWN): string|int|false {}

    /**
     * @param PgSql\Connection $connection
     * @param string|int $oid
     */
    #[Deprecated(since: '8.0', message: 'use pg_lo_create() instead')]
    function pg_locreate($connection = UNKNOWN, $oid = UNKNOWN): string|int|false {}

    /**
     * @param PgSql\Connection $connection
     * @param string|int $oid
     */
    function pg_lo_unlink($connection, $oid = UNKNOWN): bool {}

    /**
     * @param PgSql\Connection $connection
     * @param string|int $oid
     */
    #[Deprecated(since: '8.0', message: 'use pg_lo_unlink() instead')]
    function pg_lounlink($connection, $oid = UNKNOWN): bool {}

    /**
     * @param PgSql\Connection $connection
     * @param string|int $oid
     */
    function pg_lo_open($connection, $oid = UNKNOWN, string $mode = UNKNOWN): PgSql\Lob|false {}

    /**
     * @param PgSql\Connection $connection
     * @param string|int $oid
     */
    #[Deprecated(since: '8.0', message: 'use pg_lo_open() instead')]
    function pg_loopen($connection, $oid = UNKNOWN, string $mode = UNKNOWN): PgSql\Lob|false {}

    function pg_lo_close(PgSql\Lob $lob): bool {}

    #[Deprecated(since: '8.0', message: 'use pg_lo_close() instead')]
    function pg_loclose(PgSql\Lob $lob): bool {}

    function pg_lo_read(PgSql\Lob $lob, int $length = 8192): string|false {}

    #[Deprecated(since: '8.0', message: 'use pg_lo_read() instead')]
    function pg_loread(PgSql\Lob $lob, int $length = 8192): string|false {}

    function pg_lo_write(PgSql\Lob $lob, string $data, ?int $length = null): int|false {}

    #[Deprecated(since: '8.0', message: 'use pg_lo_write() instead')]
    function pg_lowrite(PgSql\Lob $lob, string $data, ?int $length = null): int|false {}

    function pg_lo_read_all(PgSql\Lob $lob): int {}

    #[Deprecated(since: '8.0', message: 'use pg_lo_read_all() instead')]
    function pg_loreadall(PgSql\Lob $lob): int {}

    /**
     * @param PgSql\Connection|string $connection
     * @param string|int $filename
     * @param string|int $oid
     */
    function pg_lo_import($connection, $filename = UNKNOWN, $oid = UNKNOWN): string|int|false {}

    /**
     * @param PgSql\Connection|string $connection
     * @param string|int $filename
     * @param string|int $oid
     */
    #[Deprecated(since: '8.0', message: 'use pg_lo_import() instead')]
    function pg_loimport($connection, $filename = UNKNOWN, $oid = UNKNOWN): string|int|false {}

    /**
     * @param PgSql\Connection|string|int $connection
     * @param string|int $oid
     * @param string|int $filename
     */
    function pg_lo_export($connection, $oid = UNKNOWN, $filename = UNKNOWN): bool {}

    /**
     * @param PgSql\Connection|string|int $connection
     * @param string|int $oid
     * @param string|int $filename
     */
    #[Deprecated(since: '8.0', message: 'use pg_lo_export() instead')]
    function pg_loexport($connection, $oid = UNKNOWN, $filename = UNKNOWN): bool {}

    function pg_lo_seek(PgSql\Lob $lob, int $offset, int $whence = SEEK_CUR): bool {}

    function pg_lo_tell(PgSql\Lob $lob): int {}

    function pg_lo_truncate(PgSql\Lob $lob, int $size): bool {}

    /** @param PgSql\Connection|int $connection */
    function pg_set_error_verbosity($connection, int $verbosity = UNKNOWN): int|false {}

    /** @param PgSql\Connection|string $connection */
    function pg_set_client_encoding($connection, string $encoding = UNKNOWN): int {}

    /**
     * @param PgSql\Connection|string $connection
     */
    #[Deprecated(since: '8.0', message: 'use pg_set_client_encoding() instead')]
    function pg_setclientencoding($connection, string $encoding = UNKNOWN): int {}

    function pg_client_encoding(?PgSql\Connection $connection = null): string {}

    #[Deprecated(since: '8.0', message: 'use pg_client_encoding() instead')]
    function pg_clientencoding(?PgSql\Connection $connection = null): string {}

    function pg_end_copy(?PgSql\Connection $connection = null): bool {}

    /** @param PgSql\Connection|string $connection */
    function pg_put_line($connection, string $query = UNKNOWN): bool {}

    /**
     * @return array<int, string>|false
     */
    function pg_copy_to(
        PgSql\Connection $connection,
        string $table_name,
        string $separator = "\t",
        string $null_as = "\\\\N",
    ): array|false {}

    function pg_copy_from(
        PgSql\Connection $connection,
        string $table_name,
        array|Traversable $rows,
        string $separator = "\t",
        string $null_as = "\\\\N",
    ): bool {}

    /**
     * @param PgSql\Connection|string $connection
     */
    function pg_escape_string($connection, string $string = UNKNOWN): string {}

    /**
     * @param PgSql\Connection|string $connection
     */
    function pg_escape_bytea($connection, string $string = UNKNOWN): string {}

    function pg_unescape_bytea(string $string): string {}

    /**
     * @param PgSql\Connection|string $connection
     */
    function pg_escape_literal($connection, string $string = UNKNOWN): string|false {}

    /**
     * @param PgSql\Connection|string $connection
     */
    function pg_escape_identifier($connection, string $string = UNKNOWN): string|false {}

    function pg_result_error(PgSql\Result $result): string|false {}

    function pg_result_error_field(PgSql\Result $result, int $field_code): string|false|null {}

    function pg_connection_status(PgSql\Connection $connection): int {}

    function pg_transaction_status(PgSql\Connection $connection): int {}

    function pg_connection_reset(PgSql\Connection $connection): bool {}

    function pg_cancel_query(PgSql\Connection $connection): bool {}

    function pg_connection_busy(PgSql\Connection $connection): bool {}

    function pg_send_query(PgSql\Connection $connection, string $query): int|bool {}

    function pg_send_query_params(PgSql\Connection $connection, string $query, array $params): int|bool {}

    function pg_send_prepare(PgSql\Connection $connection, string $statement_name, string $query): int|bool {}

    function pg_send_execute(PgSql\Connection $connection, string $statement_name, array $params): int|bool {}

    function pg_get_result(PgSql\Connection $connection): PgSql\Result|false {}

    function pg_result_status(PgSql\Result $result, int $mode = PGSQL_STATUS_LONG): string|int {}

    /**
     * @return array<int|string, int|string>
     */
    function pg_get_notify(PgSql\Connection $connection, int $mode = PGSQL_ASSOC): array|false {}

    function pg_get_pid(PgSql\Connection $connection): int {}

    /**
     * @return resource|false
     */
    function pg_socket(PgSql\Connection $connection) {}

    function pg_consume_input(PgSql\Connection $connection): bool {}

    function pg_flush(PgSql\Connection $connection): int|bool {}

    /**
     * @return array<string, array>|false
     */
    function pg_meta_data(PgSql\Connection $connection, string $table_name, bool $extended = false): array|false {}

    /**
     * @return array<string, mixed>|false
     */
    function pg_convert(PgSql\Connection $connection, string $table_name, array $values, int $flags = 0): array|false {}

    function pg_insert(
        PgSql\Connection $connection,
        string $table_name,
        array $values,
        int $flags = PGSQL_DML_EXEC,
    ): PgSql\Result|string|bool {}

    function pg_update(
        PgSql\Connection $connection,
        string $table_name,
        array $values,
        array $conditions,
        int $flags = PGSQL_DML_EXEC,
    ): string|bool {}

    function pg_delete(
        PgSql\Connection $connection,
        string $table_name,
        array $conditions,
        int $flags = PGSQL_DML_EXEC,
    ): string|bool {}

    /**
     * @return array<int, array>|string|false
     */
    function pg_select(
        PgSql\Connection $connection,
        string $table_name,
        array $conditions = [],
        int $flags = PGSQL_DML_EXEC,
        int $mode = PGSQL_ASSOC,
    ): array|string|false {}

    function pg_set_error_context_visibility(PgSql\Connection $connection, int $visibility): int {}

    function pg_result_memory_size(PgSql\Result $result): int {}

    function pg_change_password(
        PgSql\Connection $connection,
        string $user,
        #[\SensitiveParameter] string $password,
    ): bool {}

    function pg_put_copy_data(PgSql\Connection $connection, string $cmd): int {}

    function pg_put_copy_end(PgSql\Connection $connection, ?string $error = null): int {}

    /**
     * @param resource $socket
     */
    function pg_socket_poll($socket, int $read, int $write, int $timeout = -1): int {}

    function pg_set_chunked_rows_size(PgSql\Connection $connection, int $size): bool {}

    function pg_close_stmt(Pgsql\Connection $connection, string $statement_name): PgSql\Result|false {}
}

namespace PgSql {
    final class Connection {}

    final class Result {}

    final class Lob {}
}
