<?php

declare(strict_types=1);

/** @param positive-int $n */
function pos(int $n): int
{
    return $n;
}

function caller(int $x): int
{
    return pos($x);
}
