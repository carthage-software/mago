<?php

declare(strict_types=1);

function takesInt(int $n): int { return $n; }

takesInt(intval('42'));
takesInt(intval('0x1A', 16));
takesInt(intval(3.9));
takesInt(intval(true));
