<?php

declare(strict_types=1);

/**
 * @param IteratorAggregate<int, string> $agg
 *
 */
function bad(IteratorAggregate $agg): int
{
    return count($agg);
}
