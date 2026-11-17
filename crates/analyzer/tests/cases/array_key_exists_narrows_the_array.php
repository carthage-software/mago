<?php

declare(strict_types=1);

/**
 * @param non-empty-array<string, int> $values
 */
function take_non_empty_values(array $values): void
{
}

/**
 * @param array<string, int> $values
 */
function take_values(array $values): void
{
}

function literal_key_on_mixed(mixed $value): int
{
    /** @mago-expect analysis:mixed-argument */
    if (array_key_exists('a', $value)) {
        return count($value);
    }

    return 0;
}

/**
 * @param array<string, int>|null $values
 */
function variable_key_on_nullable(?array $values, string $key): void
{
    /** @mago-expect analysis:possibly-null-argument */
    if (array_key_exists($key, $values)) {
        take_non_empty_values($values);
    }
}

/**
 * @param array<string, int> $values
 */
function variable_key_else_branch(array $values, string $key): void
{
    if (array_key_exists($key, $values)) {
        take_non_empty_values($values);
    } else {
        take_values($values);
    }

    if (isset($values[$key])) {
        take_non_empty_values($values);
    }
}
