<?php

declare(strict_types=1);

/** @param 1|2|3 $n */
function takeLiteralIntAA(int $n): int
{
    return $n;
}

echo takeLiteralIntAA(1);
echo takeLiteralIntAA(3);
/** @mago-expect analysis:invalid-argument */
echo takeLiteralIntAA(0);
