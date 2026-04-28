<?php

declare(strict_types=1);

/** @mago-expect analysis:invalid-parameter-default-value */
function callables_array_default(int $n = []): int
{
    return $n;
}

echo callables_array_default(1);
