<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = 1;
$x <<= 4;
takesInt($x);  // 16

$y = 256;
$y >>= 4;
takesInt($y);  // 16
