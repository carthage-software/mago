<?php

declare(strict_types=1);

/** @param int<14, 14> $n */
function exact14(int $n): int { return $n; }

$x = (1 + 2) * 4 + 2;  // 14
exact14($x);
