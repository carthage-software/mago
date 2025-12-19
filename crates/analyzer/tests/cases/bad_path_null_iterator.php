<?php

/**
 * @mago-expect analysis:null-iterator
 * @mago-expect analysis:mixed-argument
 * @mago-expect analysis:mixed-assignment
 */
function testNullIterator(): void
{
    $items = null;
    foreach ($items as $item) {
        echo $item;
    }
}
