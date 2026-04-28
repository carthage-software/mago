<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = 5;
$y = $x ^ $x;  // always 0
takesInt($y);
