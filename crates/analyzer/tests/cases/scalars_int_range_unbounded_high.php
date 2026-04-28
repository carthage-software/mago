<?php

declare(strict_types=1);

/** @param int<0, max> $n */
function r(int $n): int { return $n; }

r(0);
r(1);
r(PHP_INT_MAX);
