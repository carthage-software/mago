<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = 0xff;
$x ^= 0x0f;
takesInt($x);  // 240
