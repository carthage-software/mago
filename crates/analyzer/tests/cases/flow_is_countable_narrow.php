<?php

declare(strict_types=1);

function flow_is_countable_narrow(mixed $v): int
{
    if (is_countable($v)) {
        return count($v);
    }

    return 0;
}
