<?php

declare(strict_types=1);

/** @param non-empty-array<string, int> $a */
function neArrayAL(array $a): int
{
    return count($a);
}

echo neArrayAL(['x' => 1]);
echo neArrayAL([]);
