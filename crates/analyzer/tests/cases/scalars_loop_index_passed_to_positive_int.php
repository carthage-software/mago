<?php

declare(strict_types=1);

/** @param positive-int $n */
function pos(int $n): int { return $n; }

for ($i = 1; $i < 10; $i++) {
    pos($i);
}
