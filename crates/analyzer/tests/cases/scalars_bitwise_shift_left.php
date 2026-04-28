<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = 1 << 4;   // 16
takesInt($x);
$y = 1 << 0;   // 1
takesInt($y);
