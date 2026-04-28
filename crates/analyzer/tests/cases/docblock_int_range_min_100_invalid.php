<?php

declare(strict_types=1);

/** @param int<min, 100> $n */
function atMostAI(int $n): int
{
    return $n;
}

/** @mago-expect analysis:invalid-argument */
atMostAI(101);
