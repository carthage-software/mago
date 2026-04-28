<?php

declare(strict_types=1);

function flow_neq_literal_null(null|int $v): int
{
    if ($v !== null) {
        return $v + 1;
    }

    return 0;
}
