<?php

declare(strict_types=1);

/** @param int<10, 100> $n */
function r(int $n): int { return $n; }

/** @mago-expect analysis:invalid-argument */
r(101);
