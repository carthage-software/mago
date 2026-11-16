<?php

declare(strict_types=1);

/**
 * @param non-falsy-string $value
 */
function take_non_falsy(string $value): void
{
}

function empty_then_zero(string $value): void
{
    if ($value !== '' && $value !== '0') {
        take_non_falsy($value);
    }
}

/**
 * @param non-empty-string $value
 */
function zero_from_non_empty(string $value): void
{
    if ($value === '0') {
        return;
    }

    take_non_falsy($value);
}

function zero_only(string $value): void
{
    if ($value !== '0') {
        take_non_falsy($value); // @mago-expect analysis:possibly-invalid-argument
    }
}
