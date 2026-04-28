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
 * @param list<string> $arr
 */
function pass_array(array $arr): void
{
    consume($arr);
}
