<?php

declare(strict_types=1);

function take_string(string $_s): void
{
}

/**
 * @param IteratorAggregate<int, string> $agg
 */
function via_iterator_iterator(IteratorAggregate $agg): void
{
    $iter = new IteratorIterator($agg);
    foreach ($iter as $v) {
        take_string($v);
    }
}
