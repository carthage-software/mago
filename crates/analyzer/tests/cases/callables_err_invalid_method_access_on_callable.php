<?php

declare(strict_types=1);

$cb = fn(int $n): int => $n;
/** @mago-expect analysis:non-existent-method */
$cb->nonExistent();
