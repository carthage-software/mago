<?php

declare(strict_types=1);

function take_int(int $_n): void {}

/**
 * @param IteratorAggregate<int, string> $agg
 */
function bad(IteratorAggregate $agg): void
{
    foreach ($agg as $_k => $v) {
        take_int($v);
    }
}
