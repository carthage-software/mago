<?php

declare(strict_types=1);

function mixed_to_int_or_mixed(mixed $value): mixed
{
    if (!\is_scalar($value) || !\is_numeric($value)) {
        return $value;
    }

    if ((string) (int) $value !== (string) $value) {
        return $value;
    }

    return (int) $value;
}

/**
 * @param numeric $value
 */
function takes_numeric(mixed $value): void
{
}

function narrow_to_numeric(mixed $value): void
{
    if (!\is_scalar($value) || !\is_numeric($value)) {
        return;
    }

    takes_numeric($value);
}
