<?php

declare(strict_types=1);

/** @param int<10, 10> $n */
function exact10(int $n): int { return $n; }

$x = 1 + 2 + 3 + 4;
exact10($x);
