<?php

declare(strict_types=1);

function take_int(int $_n): void
{
}

/**
 * @param IteratorAggregate<string, int> $agg
 */
function bad(IteratorAggregate $agg): void
{
    foreach ($agg as $k => $_v) {
        /** @mago-expect analysis:invalid-argument */
        take_int($k);
    }
}
