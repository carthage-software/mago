<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$a = intdiv(-7, 2);  // -3 (truncated toward zero, NOT floored)
takesInt($a);
