<?php

declare(strict_types=1);

/** @assert-if-true string $value */
function is_text(mixed $value): bool
{
    return is_string($value);
}

function next_value(): int|string|null
{
    return 1;
}

function while_body_and_exit(int|string $value): int
{
    while (is_string($value)) {
        $value = strlen($value);
    }

    return $value;
}

function negated_while(int|string|null $value): string
{
    while (!is_string($value)) {
        $value = next_value();
    }

    return $value;
}

function for_exit(int|string $value): int
{
    for (; is_string($value);) {
        $value = strlen($value);
    }

    return $value;
}

function do_while_exit(int|string|null $value): string
{
    do {
        $value = next_value();
    } while (!is_string($value));

    return $value;
}

function user_guard(int|string|null $value): string
{
    while (!is_text($value)) {
        $value = next_value();
    }

    return $value;
}

function body_only(int|string|null $value): int
{
    $total = 0;
    while (is_string($value)) {
        $total += strlen($value);
        $value = next_value();
    }

    return $total;
}
