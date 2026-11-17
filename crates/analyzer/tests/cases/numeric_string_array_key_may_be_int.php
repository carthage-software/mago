<?php

declare(strict_types=1);

function take_string_key(string $key): void
{
}

/**
 * @param array<int|numeric-string, int> $values
 */
function take_numeric_keys(array $values): void
{
}

/**
 * @param numeric-string $number
 */
function numeric_string_keys(string $number): void
{
    $literal = [$number => 1];
    $assigned = [];
    $assigned[$number] = 1;

    take_numeric_keys($literal);
    take_numeric_keys($assigned);

    foreach ($literal as $key => $_) {
        /** @mago-expect analysis:possibly-invalid-argument */
        take_string_key($key);
    }
}
