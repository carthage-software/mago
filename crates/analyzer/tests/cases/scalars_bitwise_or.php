<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = 0x0f | 0xf0;  // 255
takesInt($x);
$y = 5 | 3;        // 7
takesInt($y);
