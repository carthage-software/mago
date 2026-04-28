<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = 0xff ^ 0x0f;  // 240
takesInt($x);
$y = 5 ^ 3;        // 6
takesInt($y);
