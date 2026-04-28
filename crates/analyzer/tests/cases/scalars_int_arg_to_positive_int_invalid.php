<?php

declare(strict_types=1);

/** @param positive-int $n */
function pos(int $n): int { return $n; }

function caller(int $x): int {
    /** @mago-expect analysis:possibly-invalid-argument */
    return pos($x);
}
