<?php

declare(strict_types=1);

function bad(int $n): array
{
    // @mago-expect analysis:invalid-argument
    return array_keys($n);
}
