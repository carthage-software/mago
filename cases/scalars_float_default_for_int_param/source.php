<?php

declare(strict_types=1);

function bad(int $n = 1.5): int
{
    return $n;
}

bad();
