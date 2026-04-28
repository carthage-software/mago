<?php

declare(strict_types=1);

$inc = fn(int $n): int => $n + 1;
/** @mago-expect analysis:too-many-arguments */
$inc(1, 2);
