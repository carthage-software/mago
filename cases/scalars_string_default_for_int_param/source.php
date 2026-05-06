<?php

declare(strict_types=1);

function bad(int $n = 'oops'): int
{
    return $n;
}

bad();
