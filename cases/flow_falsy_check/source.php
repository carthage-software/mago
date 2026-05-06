<?php

declare(strict_types=1);

function flow_falsy_check(?int $v): int
{
    if (!$v) {
        return 0;
    }

    return $v + 1;
}
