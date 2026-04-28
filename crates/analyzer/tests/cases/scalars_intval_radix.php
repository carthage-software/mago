<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

takesInt(intval('11', 2));      // 3 (binary)
takesInt(intval('17', 8));      // 15 (octal)
takesInt(intval('ff', 16));     // 255 (hex)
takesInt(intval('1Z', 36));     // base-36
