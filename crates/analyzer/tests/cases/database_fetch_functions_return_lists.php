<?php

declare(strict_types=1);

/**
 * @return list<array<int|string, string|null>>
 */
function pgsql_fetch_all_returns_rows(PgSql\Result $result): array
{
    return pg_fetch_all($result);
}

/**
 * @return list<string|null>
 */
function pgsql_fetch_all_columns_returns_list(PgSql\Result $result): array
{
    return pg_fetch_all_columns($result);
}

/**
 * @return list<mixed>|false
 */
function mysqli_fetch_row_returns_list(mysqli_result $result): array|false
{
    return $result->fetch_row() ?? false;
}

/**
 * @return list<array<int|string, mixed>>
 */
function mysqli_fetch_all_returns_rows(mysqli_result $result): array
{
    return $result->fetch_all();
}

/**
 * @return int|numeric-string
 */
function mysqli_row_count_is_numeric(mysqli_result $result): int|string
{
    return $result->num_rows;
}

/**
 * @return non-negative-int
 */
function mysqli_param_count_is_not_negative(mysqli_stmt $statement): int
{
    return $statement->param_count;
}

function sqlite3_version_is_a_shape(): string
{
    return SQLite3::version()['versionString'];
}
