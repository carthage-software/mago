<?php

declare(strict_types=1);

/** @param non-empty-array<string, int> $a */
function neArrayAM(array $a): int
{
    return count($a);
}

neArrayAM([]);
