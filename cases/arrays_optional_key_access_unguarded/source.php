<?php

declare(strict_types=1);

/**
 * @param array{a: int, b?: string} $arr
 */
function unguarded(array $arr): string
{
    return $arr['b'];
}
