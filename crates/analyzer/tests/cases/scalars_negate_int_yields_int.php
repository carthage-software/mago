<?php

declare(strict_types=1);

/** @param int<-10, -10> $n */
function exact(int $n): int { return $n; }

$x = 10;
exact(-$x);
