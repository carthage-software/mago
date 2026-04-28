<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$x = '42';
takesInt((int) $x);
takesInt((int) 'abc');     // 0 in PHP, no error
takesInt((int) '12abc');   // 12 in PHP
takesInt((int) '');         // 0 in PHP
