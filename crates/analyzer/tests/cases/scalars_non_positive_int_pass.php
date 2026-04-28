<?php

declare(strict_types=1);

/** @param non-positive-int $n */
function np(int $n): int { return $n; }

np(0);
np(-1);
np(PHP_INT_MIN);
