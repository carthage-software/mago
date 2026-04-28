<?php

declare(strict_types=1);

function take_int(int $_n): void
{
}

function take_string(string $_s): void
{
}

/**
 * @param IteratorAggregate<int, string> $agg
 */
function iterate_aggregate(IteratorAggregate $agg): void
{
    foreach ($agg as $k => $v) {
        take_int($k);
        take_string($v);
    }
}
