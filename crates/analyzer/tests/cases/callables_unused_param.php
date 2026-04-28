<?php

declare(strict_types=1);

function callables_ignore_param(int $unused): int
{
    return 0;
}

echo callables_ignore_param(99);
