<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = 6;
$x *= 7;
takesInt($x);
