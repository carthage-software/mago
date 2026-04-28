<?php

declare(strict_types=1);

/** @param list<int> $xs */
function sumListCD(array $xs): int
{
    return array_sum($xs);
}

echo sumListCD([]);
echo sumListCD([1, 2, 3]);
