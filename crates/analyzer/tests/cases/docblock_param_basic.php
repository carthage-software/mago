<?php

declare(strict_types=1);

/**
 * @param non-empty-string $a
 * @param int<0, max> $b
 */
function take_two(string $a, int $b): string
{
    return $a . (string) $b;
}

echo take_two('hello', 0);
echo take_two('hi', 100);
/** @mago-expect analysis:invalid-argument */
echo take_two('', 0);
/** @mago-expect analysis:invalid-argument */
echo take_two('hi', -1);
