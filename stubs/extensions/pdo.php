<?php

namespace {
    class PDOException extends RuntimeException
    {
        public array|null $errorInfo;
        protected $code;
    }

    class PDO
    {
        public const PARAM_NULL = 0;

        public const PARAM_INT = 1;

        public const PARAM_STR = 2;

        public const PARAM_LOB = 3;

        public const PARAM_STMT = 4;

        public const PARAM_BOOL = 5;

        public const PARAM_STR_NATL = 1073741824;

        public const PARAM_STR_CHAR = 536870912;

        public const ATTR_DEFAULT_STR_PARAM = 21;

        public const SQLITE_DETERMINISTIC = 2048;

        public const SQLITE_OPEN_READONLY = 1;

        public const SQLITE_OPEN_READWRITE = 2;

        public const SQLITE_OPEN_CREATE = 4;

        public const SQLITE_ATTR_OPEN_FLAGS = 1000;

        public const PARAM_INPUT_OUTPUT = 2147483648;

        public const PARAM_EVT_ALLOC = 0;

        public const PARAM_EVT_FREE = 1;

        public const PARAM_EVT_EXEC_PRE = 2;

        public const PARAM_EVT_EXEC_POST = 3;

        public const PARAM_EVT_FETCH_PRE = 4;

        public const PARAM_EVT_FETCH_POST = 5;

        public const PARAM_EVT_NORMALIZE = 6;

        public const FETCH_LAZY = 1;

        public const FETCH_ASSOC = 2;

        public const FETCH_NUM = 3;

        public const FETCH_BOTH = 4;

        public const FETCH_OBJ = 5;

        public const FETCH_BOUND = 6;

        public const FETCH_COLUMN = 7;

        public const FETCH_CLASS = 8;

        public const FETCH_INTO = 9;

        public const FETCH_FUNC = 10;

        public const FETCH_GROUP = 65536;

        public const FETCH_UNIQUE = 196608;

        public const FETCH_KEY_PAIR = 12;

        public const FETCH_CLASSTYPE = 262144;

        public const FETCH_SERIALIZE = 524288;

        public const FETCH_PROPS_LATE = 1048576;

        public const FETCH_NAMED = 11;

        public const ATTR_AUTOCOMMIT = 0;

        public const ATTR_PREFETCH = 1;

        public const ATTR_TIMEOUT = 2;

        public const ATTR_ERRMODE = 3;

        public const ATTR_SERVER_VERSION = 4;

        public const ATTR_CLIENT_VERSION = 5;

        public const ATTR_SERVER_INFO = 6;

        public const ATTR_CONNECTION_STATUS = 7;

        public const ATTR_CASE = 8;

        public const ATTR_CURSOR_NAME = 9;

        public const ATTR_CURSOR = 10;

        public const ATTR_ORACLE_NULLS = 11;

        public const ATTR_PERSISTENT = 12;

        public const ATTR_STATEMENT_CLASS = 13;

        public const ATTR_FETCH_TABLE_NAMES = 14;

        public const ATTR_FETCH_CATALOG_NAMES = 15;

        public const ATTR_DRIVER_NAME = 16;

        public const ATTR_STRINGIFY_FETCHES = 17;

        public const ATTR_MAX_COLUMN_LEN = 18;

        public const ATTR_EMULATE_PREPARES = 20;

        public const ATTR_DEFAULT_FETCH_MODE = 19;

        public const ERRMODE_SILENT = 0;

        public const ERRMODE_WARNING = 1;

        public const ERRMODE_EXCEPTION = 2;

        public const CASE_NATURAL = 0;

        public const CASE_LOWER = 2;

        public const CASE_UPPER = 1;

        public const NULL_NATURAL = 0;

        public const NULL_EMPTY_STRING = 1;

        public const NULL_TO_STRING = 2;

        public const ERR_NONE = '00000';

        public const FETCH_ORI_NEXT = 0;

        public const FETCH_ORI_PRIOR = 1;

        public const FETCH_ORI_FIRST = 2;

        public const FETCH_ORI_LAST = 3;

        public const FETCH_ORI_ABS = 4;

        public const FETCH_ORI_REL = 5;

        public const FETCH_DEFAULT = 0;

        public const CURSOR_FWDONLY = 0;

        public const CURSOR_SCROLL = 1;

        public const MYSQL_ATTR_USE_BUFFERED_QUERY = 1000;

        public const MYSQL_ATTR_LOCAL_INFILE = 1001;

        public const MYSQL_ATTR_INIT_COMMAND = 1002;

        public const MYSQL_ATTR_MAX_BUFFER_SIZE = 1005;

        public const MYSQL_ATTR_READ_DEFAULT_FILE = 1003;

        public const MYSQL_ATTR_READ_DEFAULT_GROUP = 1004;

        public const MYSQL_ATTR_COMPRESS = 1003;

        public const MYSQL_ATTR_DIRECT_QUERY = 1004;

        public const MYSQL_ATTR_FOUND_ROWS = 1005;

        public const MYSQL_ATTR_IGNORE_SPACE = 1006;

        public const MYSQL_ATTR_SERVER_PUBLIC_KEY = 1012;

        public const MYSQL_ATTR_SSL_KEY = 1007;

        public const MYSQL_ATTR_SSL_CERT = 1008;

        public const MYSQL_ATTR_SSL_CA = 1009;

        public const MYSQL_ATTR_SSL_CAPATH = 1010;

        public const MYSQL_ATTR_SSL_CIPHER = 1011;

        public const MYSQL_ATTR_MULTI_STATEMENTS = 1013;

        public const MYSQL_ATTR_SSL_VERIFY_SERVER_CERT = 1014;

        public const MYSQL_ATTR_LOCAL_INFILE_DIRECTORY = 1015;

        public const PGSQL_ASSOC = 1;

        public const PGSQL_ATTR_DISABLE_PREPARES = 1000;

        public const PGSQL_BAD_RESPONSE = 5;

        public const PGSQL_BOTH = 3;

        public const PGSQL_TRANSACTION_IDLE = 0;

        public const PGSQL_TRANSACTION_ACTIVE = 1;

        public const PGSQL_TRANSACTION_INTRANS = 2;

        public const PGSQL_TRANSACTION_INERROR = 3;

        public const PGSQL_TRANSACTION_UNKNOWN = 4;

        public const PGSQL_CONNECT_ASYNC = 4;

        public const PGSQL_CONNECT_FORCE_NEW = 2;

        public const PGSQL_CONNECTION_AUTH_OK = 5;

        public const PGSQL_CONNECTION_AWAITING_RESPONSE = 4;

        public const PGSQL_CONNECTION_BAD = 1;

        public const PGSQL_CONNECTION_OK = 0;

        public const PGSQL_CONNECTION_MADE = 3;

        public const PGSQL_CONNECTION_SETENV = 6;

        public const PGSQL_CONNECTION_SSL_STARTUP = 7;

        public const PGSQL_CONNECTION_STARTED = 2;

        public const PGSQL_COMMAND_OK = 1;

        public const PGSQL_CONV_FORCE_NULL = 4;

        public const PGSQL_CONV_IGNORE_DEFAULT = 2;

        public const PGSQL_CONV_IGNORE_NOT_NULL = 8;

        public const PGSQL_COPY_IN = 4;

        public const PGSQL_COPY_OUT = 3;

        public const PGSQL_DIAG_CONTEXT = 87;

        public const PGSQL_DIAG_INTERNAL_POSITION = 112;

        public const PGSQL_DIAG_INTERNAL_QUERY = 113;

        public const PGSQL_DIAG_MESSAGE_DETAIL = 68;

        public const PGSQL_DIAG_MESSAGE_HINT = 72;

        public const PGSQL_DIAG_MESSAGE_PRIMARY = 77;

        public const PGSQL_DIAG_SEVERITY = 83;

        public const PGSQL_DIAG_SOURCE_FILE = 70;

        public const PGSQL_DIAG_SOURCE_FUNCTION = 82;

        public const PGSQL_DIAG_SOURCE_LINE = 76;

        public const PGSQL_DIAG_SQLSTATE = 67;

        public const PGSQL_DIAG_STATEMENT_POSITION = 80;

        public const PGSQL_DML_ASYNC = 1024;

        public const PGSQL_DML_EXEC = 512;

        public const PGSQL_DML_NO_CONV = 256;

        public const PGSQL_DML_STRING = 2048;

        public const PGSQL_DML_ESCAPE = 4096;

        public const PGSQL_EMPTY_QUERY = 0;

        public const PGSQL_ERRORS_DEFAULT = 1;

        public const PGSQL_ERRORS_TERSE = 0;

        public const PGSQL_ERRORS_VERBOSE = 2;

        public const PGSQL_FATAL_ERROR = 7;

        public const PGSQL_NONFATAL_ERROR = 6;

        public const PGSQL_NOTICE_ALL = 2;

        public const PGSQL_NOTICE_CLEAR = 3;

        public const PGSQL_NOTICE_LAST = 1;

        public const PGSQL_NUM = 2;

        public const PGSQL_POLLING_ACTIVE = 4;

        public const PGSQL_POLLING_FAILED = 0;

        public const PGSQL_POLLING_OK = 3;

        public const PGSQL_POLLING_READING = 1;

        public const PGSQL_POLLING_WRITING = 2;

        public const PGSQL_SEEK_CUR = 1;

        public const PGSQL_SEEK_END = 2;

        public const PGSQL_SEEK_SET = 0;

        public const PGSQL_STATUS_LONG = 1;

        public const PGSQL_STATUS_STRING = 2;

        public const PGSQL_TUPLES_OK = 2;

        public const SQLSRV_TXN_READ_UNCOMMITTED = 'READ_UNCOMMITTED';

        public const SQLSRV_TXN_READ_COMMITTED = 'READ_COMMITTED';

        public const SQLSRV_TXN_REPEATABLE_READ = 'REPEATABLE_READ';

        public const SQLSRV_TXN_SNAPSHOT = 'SNAPSHOT';

        public const SQLSRV_TXN_SERIALIZABLE = 'SERIALIZABLE';

        public const SQLSRV_ENCODING_BINARY = 2;

        public const SQLSRV_ENCODING_SYSTEM = 3;

        public const SQLSRV_ENCODING_UTF8 = 65001;

        public const SQLSRV_ENCODING_DEFAULT = 1;

        public const SQLSRV_ATTR_ENCODING = 1000;

        public const SQLSRV_ATTR_QUERY_TIMEOUT = 1001;

        public const SQLSRV_ATTR_DIRECT_QUERY = 1002;

        public const SQLSRV_ATTR_CURSOR_SCROLL_TYPE = 1003;

        public const SQLSRV_ATTR_CLIENT_BUFFER_MAX_KB_SIZE = 1004;

        public const SQLSRV_ATTR_FETCHES_NUMERIC_TYPE = 1005;

        public const SQLSRV_ATTR_FETCHES_DATETIME_TYPE = 1006;

        public const SQLSRV_ATTR_FORMAT_DECIMALS = 1007;

        public const SQLSRV_ATTR_DECIMAL_PLACES = 1008;

        public const SQLSRV_ATTR_DATA_CLASSIFICATION = 1009;

        public const SQLSRV_PARAM_OUT_DEFAULT_SIZE = -1;

        public const SQLSRV_CURSOR_KEYSET = 1;

        public const SQLSRV_CURSOR_DYNAMIC = 2;

        public const SQLSRV_CURSOR_STATIC = 3;

        public const SQLSRV_CURSOR_BUFFERED = 42;

        public const SQLITE_ATTR_READONLY_STATEMENT = 1001;

        public const SQLITE_ATTR_EXTENDED_RESULT_CODES = 1002;

        public const OCI_ATTR_ACTION = 1000;

        public const OCI_ATTR_CLIENT_INFO = 1001;

        public const OCI_ATTR_CLIENT_IDENTIFIER = 1002;

        public const OCI_ATTR_MODULE = 1003;

        public const OCI_ATTR_CALL_TIMEOUT = 1004;

        public const FB_ATTR_DATE_FORMAT = 1000;

        public const FB_ATTR_TIME_FORMAT = 1001;

        public const FB_ATTR_TIMESTAMP_FORMAT = 1002;

        /**
         * @throws PDOException
         */
        public function __construct(
            string $dsn,
            string|null $username = null,
            string|null $password = null,
            array|null $options = null,
        ) {}

        /**
         * @throws PDOException
         */
        public function prepare(string $query, array $options = []): PDOStatement|false
        {
        }

        /**
         * @throws PDOException
         */
        public function beginTransaction(): bool
        {
        }

        /**
         * @throws PDOException
         */
        public function commit(): bool
        {
        }

        /**
         * @throws PDOException
         */
        public function rollBack(): bool
        {
        }

        #[TentativeType]
        public function inTransaction(): bool
        {
        }

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function setAttribute(int $attribute, mixed $value): bool
        {
        }

        /**
         * @throws PDOException
         */
        public function exec(string $statement): int|false
        {
        }

        /**
         * @return PDOStatement|false
         *
         * @throws PDOException
         */
        public function query(string $query, int|null $fetchMode = null, mixed ...$fetchModeArgs)
        {
        }

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function lastInsertId(string|null $name = null): string|false
        {
        }

        #[TentativeType]
        public function errorCode(): null|string
        {
        }

        /**
         * @return array{0: string, 1: int, 2: string}
         */
        #[TentativeType]
        public function errorInfo(): array
        {
        }

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function getAttribute(int $attribute): mixed
        {
        }

        #[TentativeType]
        public function quote(string $string, int $type = PDO::PARAM_STR): string|false
        {
        }

        final public function __wakeup()
        {
        }

        final public function __sleep()
        {
        }

        #[TentativeType]
        public static function getAvailableDrivers(): array
        {
        }

        /**
         * @param string $function_name
         * @param callable $step_func
         * @param callable $finalize_func
         * @param int $num_args
         *
         * @return bool
         */
        public function sqliteCreateAggregate($function_name, $step_func, $finalize_func, $num_args = -1)
        {
        }

        /**
         * @param string $name
         * @param callable $callback
         *
         * @return bool
         */
        public function sqliteCreateCollation($name, $callback)
        {
        }

        /**
         * @param string $function_name
         * @param callable $callback
         * @param int $num_args
         * @param int $flags
         *
         * @return bool
         */
        public function sqliteCreateFunction($function_name, $callback, $num_args = -1, $flags = 0)
        {
        }

        /**
         * @param string $tableName
         * @param array $rows
         * @param string $separator
         * @param string $nullAs
         * @param string|null $fields
         *
         * @return bool
         */
        public function pgsqlCopyFromArray(
            $tableName,
            array $rows,
            $separator = "\t",
            $nullAs = "\\\\N",
            $fields = null,
        ) {
        }

        /**
         * @param string $tableName
         * @param string $filename
         * @param string $separator
         * @param string $nullAs
         * @param string|null $fields
         *
         * @return bool
         */
        public function pgsqlCopyFromFile($tableName, $filename, $separator = "\t", $nullAs = "\\\\N", $fields = null)
        {
        }

        /**
         * @param string $tableName
         * @param string $separator
         * @param string $nullAs
         * @param string|null $fields
         *
         * @return array|false
         */
        public function pgsqlCopyToArray($tableName, $separator = "\t", $nullAs = "\\\\N", $fields = null)
        {
        }

        /**
         * @param string $tableName
         * @param string $filename
         * @param string $separator
         * @param string $nullAs
         * @param string|null $fields
         *
         * @return bool
         */
        public function pgsqlCopyToFile($tableName, $filename, $separator = "\t", $nullAs = "\\\\N", $fields = null)
        {
        }

        /**
         * @return string|false
         */
        public function pgsqlLOBCreate()
        {
        }

        /**
         * @param string $oid
         * @param string $mode
         *
         * @return resource|false
         */
        public function pgsqlLOBOpen($oid, $mode = 'rb')
        {
        }

        /**
         * @param string $oid
         *
         * @return bool
         */
        public function pgsqlLOBUnlink($oid)
        {
        }

        /**
         * @param int $fetchMode
         * @param int $timeoutMilliseconds
         *
         * @return array|false
         */
        public function pgsqlGetNotify($fetchMode = PDO::FETCH_DEFAULT, $timeoutMilliseconds = 0)
        {
        }

        /**
         * @return int
         */
        public function pgsqlGetPid()
        {
        }

        /**
         * @throws PDOException
         */
        public static function connect(
            string $dsn,
            null|string $username = null,
            null|string $password = null,
            null|array $options = null,
        ): static {
        }
    }

    class PDOStatement implements IteratorAggregate
    {
        public string $queryString;

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function execute(array|null $params = null): bool
        {
        }

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function fetch(
            int $mode = PDO::FETCH_DEFAULT,
            int $cursorOrientation = PDO::FETCH_ORI_NEXT,
            int $cursorOffset = 0,
        ): mixed {
        }

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function bindParam(
            int|string $param,
            mixed &$var,
            int $type = PDO::PARAM_STR,
            int $maxLength = 0,
            mixed $driverOptions = null,
        ): bool {
        }

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function bindColumn(
            int|string $column,
            mixed &$var,
            int $type = PDO::PARAM_STR,
            int $maxLength = 0,
            mixed $driverOptions = null,
        ): bool {
        }

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function bindValue(int|string $param, mixed $value, int $type = PDO::PARAM_STR): bool
        {
        }

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function rowCount(): int
        {
        }

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function fetchColumn(int $column = 0): mixed
        {
        }

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function fetchAll(int $mode = PDO::FETCH_DEFAULT, mixed ...$args): array
        {
        }

        /**
         * @template T of object
         *
         * @param class-string<T>|null $class
         *
         * @return T|null
         *
         * @throws PDOException
         */
        #[TentativeType]
        public function fetchObject(string|null $class = 'stdClass', array $constructorArgs = []): object|false
        {
        }

        #[TentativeType]
        public function errorCode(): null|string
        {
        }

        /**
         * @return array{0: string, 1: int, 2: string}
         */
        #[TentativeType]
        public function errorInfo(): array
        {
        }

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function setAttribute(int $attribute, mixed $value): bool
        {
        }

        #[TentativeType]
        public function getAttribute(int $name): mixed
        {
        }

        /**
         * @throws PDOException
         */
        #[TentativeType]
        public function columnCount(): int
        {
        }

        /**
         * @return array{
         *   'name': string,
         *   'len': int,
         *   'precision': int,
         *   'oci:decl_type': intstring',
         *   'native_type': string,
         *   'scale': int,
         *   'flags': array,
         *   'pdo_type': int,
         * }|false
         */
        #[TentativeType]
        public function getColumnMeta(int $column): array|false
        {
        }

        /**
         * @param int $mode
         * @param mixed ...$args
         *
         * @return bool
         *
         * @throws PDOException
         */
        public function setFetchMode($mode, ...$args)
        {
        }

        /**
         * @throws PDOException
         */
        public function nextRowset(): bool
        {
        }

        /**
         * @throws PDOException
         */
        public function closeCursor(): bool
        {
        }

        public function debugDumpParams(): null|bool
        {
        }

        final public function __wakeup()
        {
        }

        final public function __sleep()
        {
        }

        public function getIterator(): Iterator
        {
        }

        public function connect()
        {
        }
    }

    final class PDORow
    {
        public string $queryString;
    }

    /**
     * @return array
     */
    function pdo_drivers(): array
    {
    }

    function confirm_pdo_ibm_compiled()
    {
    }
}

namespace Pdo {
    use PDO;

    class Sqlite extends PDO
    {
        public const int DETERMINISTIC = 0;
        public const int OPEN_READONLY = 1;
        public const int OPEN_READWRITE = 0;
        public const int OPEN_CREATE = 0;
        public const int ATTR_OPEN_FLAGS = 0;
        public const int ATTR_READONLY_STATEMENT = 0;
        public const int ATTR_EXTENDED_RESULT_CODES = 0;

        public function createAggregate(string $name, callable $step, callable $finalize, int $numArgs = -1): bool
        {
        }

        public function createCollation(string $name, callable $callback): bool
        {
        }

        public function createFunction(
            string $function_name,
            callable $callback,
            int $num_args = -1,
            int $flags = 0,
        ): bool {
        }

        public function loadExtension(string $name): void
        {
        }

        /** @return resource|false */
        public function openBlob(
            string $table,
            string $column,
            int $rowid,
            null|string $dbname = 'main',
            int $flags = \Pdo\Sqlite::OPEN_READONLY,
        ) {
        }
    }

    class Mysql extends PDO
    {
        public const int ATTR_USE_BUFFERED_QUERY = 0;
        public const int ATTR_LOCAL_INFILE = 0;
        public const int ATTR_INIT_COMMAND = 0;
        public const int ATTR_MAX_BUFFER_SIZE = 0;
        public const int ATTR_READ_DEFAULT_FILE = 0;
        public const int ATTR_READ_DEFAULT_GROUP = 0;
        public const int ATTR_COMPRESS = 0;
        public const int ATTR_DIRECT_QUERY = 0;
        public const int ATTR_FOUND_ROWS = 0;
        public const int ATTR_IGNORE_SPACE = 0;
        public const int ATTR_SSL_KEY = 0;
        public const int ATTR_SSL_CERT = 0;
        public const int ATTR_SSL_CA = 0;
        public const int ATTR_SSL_CAPATH = 0;
        public const int ATTR_SSL_CIPHER = 0;
        public const int ATTR_SERVER_PUBLIC_KEY = 0;
        public const int ATTR_MULTI_STATEMENTS = 0;
        public const int ATTR_SSL_VERIFY_SERVER_CERT = 0;
        public const int ATTR_LOCAL_INFILE_DIRECTORY = 0;

        public function getWarningCount(): int
        {
        }
    }

    class Pgsql extends PDO
    {
        public const int ATTR_DISABLE_PREPARES = 1000;
        public const int ATTR_RESULT_MEMORY_SIZE = 1001;
        public const int TRANSACTION_IDLE = 0;
        public const int TRANSACTION_ACTIVE = 1;
        public const int TRANSACTION_INTRANS = 2;
        public const int TRANSACTION_INERROR = 3;
        public const int TRANSACTION_UNKNOWN = 4;

        public function copyFromArray(
            string $tableName,
            array $rows,
            string $separator = "\t",
            string $nullAs = "\\\\N",
            null|string $fields = null,
        ): bool {
        }

        public function copyFromFile(
            string $tableName,
            string $filename,
            string $separator = "\t",
            string $nullAs = "\\\\N",
            null|string $fields = null,
        ): bool {
        }

        public function copyToArray(
            string $tableName,
            string $separator = "\t",
            string $nullAs = "\\\\N",
            null|string $fields = null,
        ): array|false {
        }

        public function copyToFile(
            string $tableName,
            string $filename,
            string $separator = "\t",
            string $nullAs = "\\\\N",
            null|string $fields = null,
        ): bool {
        }

        public function escapeIdentifier(string $input): string
        {
        }

        public function getNotify(int $fetchMode = \PDO::FETCH_DEFAULT, int $timeoutMilliseconds = 0): array|false
        {
        }

        public function getPid(): int
        {
        }

        public function lobCreate(): string|false
        {
        }

        /**
         * @return resource|false
         */
        public function lobOpen(string $oid, string $mode = 'rb')
        {
        }

        public function lobUnlink(string $oid): bool
        {
        }

        public function setNoticeCallback(null|callable $callback): void
        {
        }
    }
}
