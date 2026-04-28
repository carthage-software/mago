<?php

declare(strict_types=1);

/** @param negative-int $n */
function neg(int $n): int { return $n; }

/** @mago-expect analysis:invalid-argument */
neg(0);
