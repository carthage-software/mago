<?php

declare(strict_types=1);

/**
 * @param array<string, int> $a
 * @param array<string, int> $b
 * @return array<string, int>
 */
function replace_deep(array $a, array $b): array
{
    return array_replace_recursive($a, $b);
}
