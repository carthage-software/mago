<?php

declare(strict_types=1);

function to_number(mixed $value): int|float|string
{
    if (is_numeric($value)) {
        return $value;
    }

    return 0;
}

function takes_scalar(int|float|string $value): void
{
    echo $value;
}

function pass_numeric(mixed $value): void
{
    if (is_numeric($value)) {
        takes_scalar($value);
    }
}

/** @param numeric $value */
function numeric_string_branch(mixed $value): string
{
    if (is_string($value)) {
        return $value;
    }

    return (string) $value;
}

/**
 * @param numeric $value
 *
 * @return numeric-string
 */
function only_numeric_strings(mixed $value): string
{
    if (!is_string($value)) {
        return '0';
    }

    return $value;
}

/** @param numeric $value */
function not_a_plain_int(mixed $value): int
{
    // @mago-expect analysis:invalid-return-statement
    return $value;
}
