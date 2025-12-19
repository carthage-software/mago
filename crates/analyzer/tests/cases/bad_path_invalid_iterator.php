<?php

/**
 * @mago-expect analysis:invalid-iterator
 * @mago-expect analysis:mixed-argument
 * @mago-expect analysis:mixed-assignment
 */
function testInvalidIterator(): void
{
    $notIterable = 42;
    foreach ($notIterable as $item) {
        echo $item;
    }
}
