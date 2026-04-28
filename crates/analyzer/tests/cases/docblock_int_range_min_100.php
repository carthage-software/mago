<?php

declare(strict_types=1);

/** @param int<min, 100> $n */
function atMostAH(int $n): int
{
    return $n;
}

atMostAH(0);
atMostAH(100);
atMostAH(-50);
/** @mago-expect analysis:invalid-argument */
atMostAH(101);
