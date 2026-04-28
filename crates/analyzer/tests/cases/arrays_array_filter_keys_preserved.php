<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function consume(array $xs): void
{
    foreach (array_filter($xs) as $i => $v) {
        echo $i, '=', $v;
    }
}
