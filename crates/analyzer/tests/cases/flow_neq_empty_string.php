<?php

declare(strict_types=1);

function flow_neq_empty_string(string $s): int
{
    if ($s !== '') {
        return strlen($s);
    }

    return 0;
}
