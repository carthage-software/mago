<?php

declare(strict_types=1);

/**
 * @return non-empty-string
 */
function text(mixed $m): string
{
    if ($m) {
        if (is_string($m)) {
            return $m;
        }
    }

    return 'none';
}

/**
 * @return non-empty-array<array-key, mixed>
 */
function items(mixed $m): array
{
    if (!empty($m)) {
        if (is_array($m)) {
            return $m;
        }
    }

    return ['none'];
}

/**
 * @return true
 */
function flag(mixed $m): bool
{
    if (!$m) {
        return true;
    }

    if (is_bool($m)) {
        return $m;
    }

    return true;
}
