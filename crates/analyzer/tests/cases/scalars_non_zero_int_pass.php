<?php

declare(strict_types=1);

/** @param non-zero-int $n */
function nz(int $n): int { return $n; }

nz(1);
nz(-1);
nz(42);
nz(-42);
