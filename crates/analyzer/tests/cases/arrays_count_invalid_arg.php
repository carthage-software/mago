<?php

declare(strict_types=1);

function bad(int $n): int
{
    // @mago-expect analysis:invalid-argument
    return count($n);
}
