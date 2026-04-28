<?php

declare(strict_types=1);

/** @param non-zero-int $n */
function nz(int $n): int { return $n; }

/** @mago-expect analysis:invalid-argument */
nz(0);
