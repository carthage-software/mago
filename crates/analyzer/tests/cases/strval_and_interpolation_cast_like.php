<?php

declare(strict_types=1);

/**
 * @param numeric-string $value
 */
function strval_cast_numeric(string $value): string
{
    return $value;
}

/**
 * @param lowercase-string $value
 */
function strval_cast_lower(string $value): string
{
    return $value;
}

/**
 * @param non-empty-lowercase-string $low
 */
function strval_cast_run(int $int, string $low): void
{
    strval_cast_numeric((string) $int);
    strval_cast_numeric(strval($int));
    strval_cast_numeric("$int");
    strval_cast_lower("$low");
}
