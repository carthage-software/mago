<?php

declare(strict_types=1);

const DEFAULTS = ['name' => 'default'];

/** @return array<string, int> */
function values(): array
{
    return ['count' => 1];
}

function from_literal(): int
{
    return extract(['a' => 1]);
}

function from_constant(): int
{
    return extract(DEFAULTS);
}

function from_call(): int
{
    return extract(values());
}

/**
 * @param array<string, int> $values
 *
 * @return array<string, int>
 */
function keeps_variable_type(array $values): array
{
    extract($values);

    return $values;
}
