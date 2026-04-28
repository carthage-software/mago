<?php

declare(strict_types=1);

/**
 * @return array{a: int, b?: string}
 */
function maybe_b(): array
{
    return ['a' => 1];
}

/**
 * @param array{a: int, b?: string} $arr
 */
function read_b(array $arr): string
{
    if (isset($arr['b'])) {
        return $arr['b'];
    }

    return '';
}
