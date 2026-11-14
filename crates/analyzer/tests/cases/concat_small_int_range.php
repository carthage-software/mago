<?php

declare(strict_types=1);

/**
 * @param 0|1|2 $k
 * @return 'item0'|'item1'|'item2'
 */
function concat_range_from_literals(int $k): string
{
    return 'item' . $k;
}

/**
 * @param int<0, 2> $k
 * @return 'item0'|'item1'|'item2'
 */
function concat_range_from_range(int $k): string
{
    return 'item' . $k;
}
