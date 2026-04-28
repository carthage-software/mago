<?php

declare(strict_types=1);

/** @param positive-int $n */
function defaultPosCI(int $n = 1): int
{
    return $n;
}

echo defaultPosCI();
echo defaultPosCI(5);
