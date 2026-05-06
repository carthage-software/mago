<?php

declare(strict_types=1);

function takes_ints(int ...$xs): int
{
    $total = 0;
    foreach ($xs as $x) {
        $total += $x;
    }
    return $total;
}

/**
 * @param list<int> $xs
 */
function call_via_spread(array $xs): int
{
    return takes_ints(...$xs);
}
