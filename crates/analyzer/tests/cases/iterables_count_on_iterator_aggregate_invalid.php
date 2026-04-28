<?php

declare(strict_types=1);

/**
 * @param IteratorAggregate<int, string> $agg
 *
 * @mago-expect analysis:possibly-invalid-argument
 */
function bad(IteratorAggregate $agg): int
{
    return count($agg);
}
