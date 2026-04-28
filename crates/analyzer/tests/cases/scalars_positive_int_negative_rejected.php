<?php

declare(strict_types=1);

/** @param positive-int $n */
function pos(int $n): int { return $n; }

/** @mago-expect analysis:invalid-argument */
pos(-1);
