<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 * @return array<string, string>
 */
function stringify(array $arr): array
{
    return array_map(static fn(int $v): string => (string) $v, $arr);
}
