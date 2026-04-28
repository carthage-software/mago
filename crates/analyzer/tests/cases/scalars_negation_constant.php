<?php

declare(strict_types=1);

/** @param int<-7, -7> $n */
function exact(int $n): int { return $n; }

const SEVEN = 7;
$x = -SEVEN;
exact($x);
