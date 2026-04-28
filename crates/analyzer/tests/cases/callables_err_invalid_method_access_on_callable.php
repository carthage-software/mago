<?php

declare(strict_types=1);

$cb = fn(int $n): int => $n;
/** @mago-expect analysis:invalid-method-access */
$cb->nonExistent();
