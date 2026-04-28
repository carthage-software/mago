<?php

declare(strict_types=1);

function flow_eq_literal_true(bool $b): int
{
    if ($b === true) {
        return 1;
    }

    return 0;
}
