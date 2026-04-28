<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

takesInt((int) true);   // 1
takesInt((int) false);  // 0
