<?php

declare(strict_types=1);

/** @mago-expect analysis:invalid-parameter-default-value */
function callables_bad_default(int $n = 'not int'): int
{
    return $n;
}

echo callables_bad_default(1);
