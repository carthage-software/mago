<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

takesInt(min(1, 2, 3));
takesInt(max(1, 2, 3));
takesInt(min(-5, -3));
takesInt(max(PHP_INT_MIN, 0));
