<?php

declare(strict_types=1);

/** @param int<10, 20> $n */
function r(int $n): int { return $n; }

/** @param int<30, 40> $x */
function caller(int $x): int {
    /** @mago-expect analysis:invalid-argument */
    return r($x);
}
