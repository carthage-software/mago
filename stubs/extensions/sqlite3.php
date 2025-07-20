<?php

/**
 * @var int
 */
const SQLITE3_ASSOC = UNKNOWN;

/**
 * @var int
 */
const SQLITE3_NUM = UNKNOWN;

/**
 * @var int
 */
const SQLITE3_BOTH = UNKNOWN;

/**
 * @var int
 */
const SQLITE3_INTEGER = UNKNOWN;

/**
 * @var int
 */
const SQLITE3_FLOAT = UNKNOWN;

/**
 * @var int
 */
const SQLITE3_TEXT = UNKNOWN;

/**
 * @var int
 */
const SQLITE3_BLOB = UNKNOWN;

/**
 * @var int
 */
const SQLITE3_NULL = UNKNOWN;

/**
 * @var int
 */
const SQLITE3_OPEN_READONLY = UNKNOWN;

/**
 * @var int
 */
const SQLITE3_OPEN_READWRITE = UNKNOWN;

/**
 * @var int
 */
const SQLITE3_OPEN_CREATE = UNKNOWN;

/**
 * @var int
 */
const SQLITE3_DETERMINISTIC = UNKNOWN;

class SQLite3Exception extends \Exception
{
}

class SQLite3
{
    public const int OK = UNKNOWN;
    public const int DENY = UNKNOWN;
    public const int IGNORE = UNKNOWN;
    public const int CREATE_INDEX = UNKNOWN;
    public const int CREATE_TABLE = UNKNOWN;
    public const int CREATE_TEMP_INDEX = UNKNOWN;
    public const int CREATE_TEMP_TABLE = UNKNOWN;
    public const int CREATE_TEMP_TRIGGER = UNKNOWN;
    public const int CREATE_TEMP_VIEW = UNKNOWN;
    public const int CREATE_TRIGGER = UNKNOWN;
    public const int CREATE_VIEW = UNKNOWN;
    public const int DELETE = UNKNOWN;
    public const int DROP_INDEX = UNKNOWN;
    public const int DROP_TABLE = UNKNOWN;
    public const int DROP_TEMP_INDEX = UNKNOWN;
    public const int DROP_TEMP_TABLE = UNKNOWN;
    public const int DROP_TEMP_TRIGGER = UNKNOWN;
    public const int DROP_TEMP_VIEW = UNKNOWN;
    public const int DROP_TRIGGER = UNKNOWN;
    public const int DROP_VIEW = UNKNOWN;
    public const int INSERT = UNKNOWN;
    public const int PRAGMA = UNKNOWN;
    public const int READ = UNKNOWN;
    public const int SELECT = UNKNOWN;
    public const int TRANSACTION = UNKNOWN;
    public const int UPDATE = UNKNOWN;
    public const int ATTACH = UNKNOWN;
    public const int DETACH = UNKNOWN;
    public const int ALTER_TABLE = UNKNOWN;
    public const int REINDEX = UNKNOWN;
    public const int ANALYZE = UNKNOWN;
    public const int CREATE_VTABLE = UNKNOWN;
    public const int DROP_VTABLE = UNKNOWN;
    public const int FUNCTION = UNKNOWN;
    public const int SAVEPOINT = UNKNOWN;
    public const int COPY = UNKNOWN;
    public const int RECURSIVE = UNKNOWN;

    public function __construct(
        string $filename,
        int $flags = SQLITE3_OPEN_READWRITE | SQLITE3_OPEN_CREATE,
        string $encryptionKey = '',
    ) {}

    public function open(
        string $filename,
        int $flags = SQLITE3_OPEN_READWRITE | SQLITE3_OPEN_CREATE,
        string $encryptionKey = '',
    ): void {
    }

    public function close(): bool
    {
    }

    public static function version(): array
    {
    }

    public function lastInsertRowID(): int
    {
    }

    public function lastErrorCode(): int
    {
    }

    public function lastExtendedErrorCode(): int
    {
    }

    public function lastErrorMsg(): string
    {
    }

    public function changes(): int
    {
    }

    public function busyTimeout(int $milliseconds): bool
    {
    }

    public function loadExtension(string $name): bool
    {
    }

    public function backup(
        SQLite3 $destination,
        string $sourceDatabase = 'main',
        string $destinationDatabase = 'main',
    ): bool {
    }

    public static function escapeString(string $string): string
    {
    }

    public function prepare(string $query): SQLite3Stmt|false
    {
    }

    public function exec(string $query): bool
    {
    }

    public function query(string $query): SQLite3Result|false
    {
    }

    public function querySingle(string $query, bool $entireRow = false): mixed
    {
    }

    public function createFunction(string $name, callable $callback, int $argCount = -1, int $flags = 0): bool
    {
    }

    public function createAggregate(
        string $name,
        callable $stepCallback,
        callable $finalCallback,
        int $argCount = -1,
    ): bool {
    }

    public function createCollation(string $name, callable $callback): bool
    {
    }

    /** @return resource|false */
    public function openBlob(
        string $table,
        string $column,
        int $rowid,
        string $database = 'main',
        int $flags = SQLITE3_OPEN_READONLY,
    ) {
    }

    public function enableExceptions(bool $enable = false): bool
    {
    }

    public function enableExtendedResultCodes(bool $enable = true): bool
    {
    }

    public function setAuthorizer(null|callable $callback): bool
    {
    }
}

class SQLite3Stmt
{
    private function __construct(SQLite3 $sqlite3, string $query) {}

    public function bindParam(string|int $param, mixed &$var, int $type = SQLITE3_TEXT): bool
    {
    }

    public function bindValue(string|int $param, mixed $value, int $type = SQLITE3_TEXT): bool
    {
    }

    public function clear(): bool
    {
    }

    public function close(): true
    {
    }

    public function execute(): SQLite3Result|false
    {
    }

    public function getSQL(bool $expand = false): string|false
    {
    }

    public function paramCount(): int
    {
    }

    public function readOnly(): bool
    {
    }

    public function reset(): bool
    {
    }

    public function busy(): bool
    {
    }

    public const int EXPLAIN_MODE_PREPARED = 0;
    public const int EXPLAIN_MODE_EXPLAIN = 1;
    public const int EXPLAIN_MODE_EXPLAIN_QUERY_PLAN = 2;

    public function explain(): int
    {
    }

    public function setExplain(int $mode): bool
    {
    }
}

class SQLite3Result
{
    private function __construct() {}

    public function numColumns(): int
    {
    }

    public function columnName(int $column): string|false
    {
    }

    public function columnType(int $column): int|false
    {
    }

    public function fetchArray(int $mode = SQLITE3_BOTH): array|false
    {
    }

    public function fetchAll(int $mode = SQLITE3_BOTH): array|false
    {
    }

    public function reset(): bool
    {
    }

    public function finalize(): true
    {
    }
}
