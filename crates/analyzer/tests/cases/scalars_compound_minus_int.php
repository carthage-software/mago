<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = 10;
$x -= 4;
takesInt($x);
