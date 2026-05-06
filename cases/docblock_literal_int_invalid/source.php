<?php

declare(strict_types=1);

/** @param 1|2|3 $n */
function takeLiteralIntAB(int $n): int
{
    return $n;
}

echo takeLiteralIntAB(4);
