<?php

declare(strict_types=1);

/** @param 1|2|3 $n */
function takeLiteralIntAB(int $n): int
{
    return $n;
}

/** @mago-expect analysis:invalid-argument */
echo takeLiteralIntAB(4);
