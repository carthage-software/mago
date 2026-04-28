<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$a = -7 % 3;   // -1 (sign follows dividend in PHP)
$b = 7 % -3;   // 1
takesInt($a);
takesInt($b);
