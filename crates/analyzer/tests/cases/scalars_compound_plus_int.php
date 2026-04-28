<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = 5;
$x += 3;
takesInt($x);
