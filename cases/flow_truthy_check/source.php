<?php

declare(strict_types=1);

function flow_truthy_check(?string $v): int
{
    if ($v) {
        return strlen($v);
    }

    return 0;
}
