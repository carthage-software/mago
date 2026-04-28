<?php

declare(strict_types=1);

/** @param non-empty-list<int> $a */
function neListAN(array $a): int
{
    return $a[0];
}

echo neListAN([1, 2, 3]);
/** @mago-expect analysis:possibly-invalid-argument */
echo neListAN([]);
