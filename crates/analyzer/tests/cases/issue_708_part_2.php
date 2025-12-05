<?php

declare(strict_types=1);

/**
 * @param list<string>|string|null $value
 */
function check1(array|string|null $value): string|null
{
    if ($value === null || is_array($value) && count($value) === 0) {
        return null;
    }

    return 'test';
}

/**
 * @param array<string>|string|null $value
 */
function check2(array|string|null $value): string|null
{
    if ($value === null || is_array($value) && count($value) === 0) {
        return null;
    }

    return 'test';
}
