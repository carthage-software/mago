<?php

declare(strict_types=1);

/** @param int<6, 6> $n */
function exact6(int $n): int { return $n; }

$a = 2 + 4;
exact6($a);
