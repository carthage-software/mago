<?php

declare(strict_types=1);

function parse_integer(int|string $value): int
{
    if (is_int($value)) {
        return $value;
    }

    if (ctype_digit($value)) {
        return (int) $value;
    }

    return 0;
}

function test(): void
{
    echo parse_integer(100) . "\n";
    echo parse_integer('200') . "\n";
    echo parse_integer('abc') . "\n";
}
