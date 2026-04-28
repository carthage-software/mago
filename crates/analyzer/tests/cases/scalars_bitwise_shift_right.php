<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = 16 >> 2;  // 4
takesInt($x);
$y = -8 >> 1;  // -4 (arithmetic shift)
takesInt($y);
