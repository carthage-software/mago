<?php

declare(strict_types=1);

$double_int = fn(int $n): int => $n * 2;
$double_int('not int');
