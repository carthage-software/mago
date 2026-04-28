<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = ~5;   // -6
takesInt($x);
$y = ~0;   // -1
takesInt($y);
