<?php

declare(strict_types=1);

function and_assignment_source(): ?string
{
    return null;
}

function and_assignment_existing_variable(?string $value): bool
{
    return ($value = and_assignment_source()) !== null && strlen($value) > 1;
}

function and_coalesce_assignment_existing_variable(?string $value, ?string $fallback): bool
{
    return null !== ($value ??= $fallback) && strlen($value) > 1;
}

function and_assignment_after_null_check(?string $value): bool
{
    return $value === null && ($value = and_assignment_source()) !== null && strlen($value) > 1;
}

/**
 * @param resource $handle
 */
function and_assignment_in_loop_condition($handle): int
{
    $line = '';
    $count = 0;
    while (($line = fgets($handle)) !== false && strlen($line) > 1) {
        $count++;
    }

    return $count;
}
