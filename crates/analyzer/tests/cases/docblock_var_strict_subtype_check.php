<?php

declare(strict_types=1);

function get_int_or_string(): int|string
{
    return 1;
}

function use_var_strict(): void
{
    $x = get_int_or_string();
    /** @var int $x */
    echo $x + 1;
}

use_var_strict();
