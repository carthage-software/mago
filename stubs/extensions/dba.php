<?php

namespace Dba {
    final class Connection
    {
    }
}

namespace {
    /**
     * @var int
     */
    const DBA_LMDB_USE_SUB_DIR = 0;

    /**
     * @var int
     */
    const DBA_LMDB_NO_SUB_DIR = UNKNOWN;

    function dba_popen(
        string $path,
        string $mode,
        null|string $handler = null,
        int $permission = 0o644,
        int $map_size = 0,
        null|int $flags = null,
    ): Dba\Connection|false {
    }

    function dba_open(
        string $path,
        string $mode,
        null|string $handler = null,
        int $permission = 0o644,
        int $map_size = 0,
        null|int $flags = null,
    ): Dba\Connection|false {
    }

    function dba_close(Dba\Connection $dba): void
    {
    }

    function dba_exists(string|array $key, Dba\Connection $dba): bool
    {
    }

    /**
     * @param Dba\Connection|int $dba
     * @param Dba\Connection|int $skip
     */
    function dba_fetch(string|array $key, $dba, $skip = 0): string|false
    {
    }

    /**
     * @return array<int, string>|false
     */
    function dba_key_split(string|false|null $key): array|false
    {
    }

    function dba_firstkey(Dba\Connection $dba): string|false
    {
    }

    function dba_nextkey(Dba\Connection $dba): string|false
    {
    }

    function dba_delete(string|array $key, Dba\Connection $dba): bool
    {
    }

    function dba_insert(string|array $key, string $value, Dba\Connection $dba): bool
    {
    }

    function dba_replace(string|array $key, string $value, Dba\Connection $dba): bool
    {
    }

    function dba_optimize(Dba\Connection $dba): bool
    {
    }

    function dba_sync(Dba\Connection $dba): bool
    {
    }

    /**
     * @return array<int|string, string>
     */
    function dba_handlers(bool $full_info = false): array
    {
    }

    /**
     * @return array<int, string>
     */
    function dba_list(): array
    {
    }
}
