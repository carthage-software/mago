<?php

declare(strict_types=1);

/** @param int<0, max> $n */
function nonNegAG(int $n): int
{
    return $n;
}

/** @mago-expect analysis:invalid-argument */
nonNegAG(-1);
