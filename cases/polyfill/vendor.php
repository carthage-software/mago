<?php

// Polyfill for array_any function, which is available in PHP 8.3 and later.

if (!function_exists('array_any')) {
    function array_any(array $array, callable $callback): bool
    {
        foreach ($array as $item) {
            if ($callback($item)) {
                return true;
            }
        }

        return false;
    }
}
