<?php

declare(strict_types=1);

/**
 * @param Iterator<int, string> $iter
 */
function consume(Iterator $iter): void
{
    foreach ($iter as $v) {
        echo $v;
    }
}

/**
 * @param list<string> $arr
 */
function bad(array $arr): void
{
    /** @mago-expect analysis:invalid-argument */
    consume($arr);
}
