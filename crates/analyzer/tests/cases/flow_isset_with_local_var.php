<?php

declare(strict_types=1);

function maybe_int(): null|int
{
    return null;
}

function flow_isset_with_local_var(): int
{
    $x = maybe_int();

    if (isset($x)) {
        return $x + 1;
    }

    return 0;
}
