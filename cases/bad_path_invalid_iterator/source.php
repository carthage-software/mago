<?php

/**
 */
function testInvalidIterator(): void
{
    $notIterable = 42;
    foreach ($notIterable as $item) {
        echo $item;
    }
}
