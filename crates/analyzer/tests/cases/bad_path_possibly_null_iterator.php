<?php

/**
 * @param list<int>|null $items
 * @mago-expect analysis:possibly-null-iterator
 */
function testPossiblyNullIterator(?array $items): void
{
    foreach ($items as $item) {
        echo $item;
    }
}
