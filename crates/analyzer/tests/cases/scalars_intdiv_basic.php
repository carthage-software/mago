<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$a = intdiv(10, 3);  // 3
takesInt($a);
$b = intdiv(-10, 3); // -3 (truncated toward zero)
takesInt($b);
$c = intdiv(7, 2);   // 3
takesInt($c);
