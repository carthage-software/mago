<?php

declare(strict_types=1);

/** @param non-negative-int $n */
function nn(int $n): int { return $n; }

function example(string $s): int {
    return nn(abs(intval($s)));
}

example('-5');
