<?php

declare(strict_types=1);

/** @param negative-int $n */
function neg(int $n): int { return $n; }

function caller(int $x): int {
    /** @mago-expect analysis:possibly-invalid-argument */
    return neg($x);
}
