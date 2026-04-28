<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$a = 10 % 3;   // 1
takesInt($a);
$b = -10 % 3;  // -1 (PHP keeps the sign of dividend)
takesInt($b);
