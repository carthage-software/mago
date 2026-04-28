<?php

declare(strict_types=1);

/** @param positive-int $n */
function pos(int $n): int { return $n; }

pos(1);
pos(42);
pos(PHP_INT_MAX);
