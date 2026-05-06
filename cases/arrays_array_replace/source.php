<?php

declare(strict_types=1);

/**
 * @param array<string, int> $a
 * @param array<string, int> $b
 * @return array<string, int>
 */
function replace_with(array $a, array $b): array
{
    return array_replace($a, $b);
}
