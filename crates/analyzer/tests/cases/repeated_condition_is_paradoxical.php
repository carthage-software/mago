<?php

declare(strict_types=1);

function repeated_or(string $value): int
{
    if ($value === 'a' || $value === 'b') {
        return 1;
    }

    // @mago-expect analysis:paradoxical-condition
    if ($value === 'a' || $value === 'b') {
        return 2;
    }

    return 3;
}

function repeated_instanceof(object $value): int
{
    if ($value instanceof Countable) {
        return 1;
    }

    // @mago-expect analysis:paradoxical-condition
    if ($value instanceof Countable) {
        return 2;
    }

    return 3;
}

/**
 * @param list<string> $opening
 * @param list<string> $closing
 */
function different_arrays(string $code, array $opening, array $closing): int
{
    if (in_array($code, $opening, true)) {
        return 1;
    } elseif (in_array($code, $closing, true)) {
        return 2;
    }

    return 3;
}
