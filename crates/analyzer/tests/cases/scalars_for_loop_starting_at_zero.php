<?php

declare(strict_types=1);

/** @param positive-int $n */
function pos(int $n): int { return $n; }

for ($i = 0; $i < 10; $i++) {
    /** @mago-expect analysis:possibly-invalid-argument */
    pos($i);
}
