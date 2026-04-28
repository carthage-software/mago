<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

$arr = [1, 2, 3];
takesInt($arr[0]);
takesInt($arr[1]);
takesInt($arr[2]);
