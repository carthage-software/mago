<?php

declare(strict_types=1);

$cb = fn(int $n): int => $n;
$cb->nonExistent();
