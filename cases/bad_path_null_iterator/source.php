<?php

/**
 */
function testNullIterator(): void
{
    $items = null;
    foreach ($items as $item) {
        echo $item;
    }
}
