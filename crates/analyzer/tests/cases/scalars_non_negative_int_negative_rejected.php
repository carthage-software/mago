<?php

declare(strict_types=1);

/** @param non-negative-int $n */
function nn(int $n): int { return $n; }

/** @mago-expect analysis:invalid-argument */
nn(-1);
