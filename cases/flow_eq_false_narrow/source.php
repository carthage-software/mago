<?php

declare(strict_types=1);

function flow_eq_false_narrow(bool $b): int
{
    if ($b === false) {
        return 0;
    }

    return 1;
}
