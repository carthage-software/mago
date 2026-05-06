<?php

declare(strict_types=1);

/**
 * @param iterable<int, string> $it
 */
function consume(iterable $it): void
{
    foreach ($it as $v) {
        echo $v;
    }
}

/**
 * @param Iterator<int, string> $iter
 */
function pass_iterator(Iterator $iter): void
{
    consume($iter);
}
