<?php

declare(strict_types=1);

/** @param negative-int $n */
function neg(int $n): int { return $n; }

neg(-1);
neg(-42);
neg(PHP_INT_MIN);
