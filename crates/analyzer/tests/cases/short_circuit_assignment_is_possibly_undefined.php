<?php

declare(strict_types=1);

function short_circuit_and(bool $enabled, string $value): void
{
    $result = $enabled && ($assigned = $value);
    /** @mago-expect analysis:possibly-undefined-variable */
    echo $assigned;
}

function short_circuit_or(bool $enabled, string $value): bool
{
    /** @mago-expect analysis:possibly-undefined-variable */
    return ($enabled || ($assigned = $value)) && $assigned === 'a';
}

function short_circuit_in_condition(bool $enabled, string $value): string
{
    if ($enabled && ($assigned = $value)) {
        return $assigned;
    }

    return $enabled && ($other = $value) ? $other : '';
}
