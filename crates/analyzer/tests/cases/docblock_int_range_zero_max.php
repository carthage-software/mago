<?php

declare(strict_types=1);

/** @param int<0, max> $n */
function nonNegAF(int $n): int
{
    return $n;
}

nonNegAF(0);
nonNegAF(99);
/** @mago-expect analysis:invalid-argument */
nonNegAF(-1);
