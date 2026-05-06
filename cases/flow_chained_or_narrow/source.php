<?php

declare(strict_types=1);

function flow_chained_or_narrow(int $v): string
{
    if ($v === 1 || $v === 2 || $v === 3) {
        return 'low';
    }

    return 'other';
}
