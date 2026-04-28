<?php

declare(strict_types=1);

final class FailE extends RuntimeException
{
}

/**
 * @throws FailE
 */
function might_fail(int $x): int
{
    if ($x < 0) {
        throw new FailE('negative');
    }

    return $x;
}

function caller_handles(): int
{
    try {
        return might_fail(-1);
    } catch (FailE $e) {
        return 0;
    }
}

echo caller_handles();
