<?php

declare(strict_types=1);

function elseif_defines_variable_when_if_exits(bool $first, bool $second): int
{
    if ($first) {
        return 0;
    } elseif ($second) {
        $value = 1;
    } else {
        return -1;
    }

    return $value;
}

function elseif_without_assignment_leaves_variable_undefined(bool $first, bool $second): int
{
    if ($first) {
        return 0;
    } elseif ($second) {
        $other = 1;
    } else {
        $value = 1;
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    return $value;
}
