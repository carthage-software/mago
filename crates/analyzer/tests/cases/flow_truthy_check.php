<?php

declare(strict_types=1);

function flow_truthy_check(null|string $v): int
{
    if ($v) {
        return strlen($v);
    }

    return 0;
}
