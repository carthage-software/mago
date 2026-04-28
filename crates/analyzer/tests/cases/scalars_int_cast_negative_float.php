<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$a = (int) -1.7;  // -1 (truncated toward zero)
takesInt($a);
