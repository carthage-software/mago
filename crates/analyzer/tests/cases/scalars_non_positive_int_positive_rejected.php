<?php

declare(strict_types=1);

/** @param non-positive-int $n */
function np(int $n): int { return $n; }

/** @mago-expect analysis:invalid-argument */
np(1);
