<?php

declare(strict_types=1);

/** @param int<6, max> $i */
function range_outside(int $i): mixed
{
    $list = [1, 21, 22];

    // @mago-expect analysis:undefined-int-array-index
    return $list[$i];
}

/** @param 7|8 $i */
function literals_outside(int $i): mixed
{
    $list = [1, 21, 22];

    // @mago-expect analysis:undefined-int-array-index
    return $list[$i];
}

/** @param 1|2 $i */
function literals_inside(int $i): int
{
    $list = [1, 21, 22];

    return $list[$i];
}
