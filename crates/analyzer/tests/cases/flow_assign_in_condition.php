<?php

declare(strict_types=1);

function maybe_value(): null|string
{
    return null;
}

function flow_assign_in_condition(): int
{
    if (($v = maybe_value()) !== null) {
        return strlen($v);
    }

    return 0;
}
