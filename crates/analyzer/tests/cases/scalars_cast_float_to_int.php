<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = 1.9;
takesInt((int) $x);
takesInt((int) -1.9);
takesInt((int) 0.0);
