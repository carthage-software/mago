<?php

/**
 * @param list<int>|null $items
 */
function testPossiblyNullIterator(?array $items): void
{
    foreach ($items as $item) {
        echo $item;
    }
}
