<?php

/**
 * @param array<int|string, array{float, string}> $cache
 *
 * @mago-expect analysis:possibly-invalid-operand
 */
function cache_put(array &$cache, int $max, int|string $key, string $val): void
{
    if (count($cache) >= $max) {
        $low_val = -1;
        $low_idx = -1;
        foreach ($cache as $k => $v) {
            if ($v[0] < $low_val || $low_val === -1) {
                $low_idx = $k;
                $low_val = $v;
            }
        }

        unset($cache[$low_idx]);
    }

    $cache[$key] = [microtime(true), $val];
}

function identity_is_silent(array|string $value): bool
{
    return $value !== '' && $value !== [];
}
