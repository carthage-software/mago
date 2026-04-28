<?php

declare(strict_types=1);

/** @param int<24, 24> $n */
function exact24(int $n): int { return $n; }

$a = 6 * 4;
exact24($a);
