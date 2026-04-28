<?php

declare(strict_types=1);

/**
 * @param IteratorAggregate<int, string> $agg
 *
 * @return Generator<int, string>
 */
function relay(IteratorAggregate $agg): Generator
{
    yield from $agg;
}
