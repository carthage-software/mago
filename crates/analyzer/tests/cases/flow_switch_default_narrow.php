<?php

declare(strict_types=1);

function flow_switch_default_narrow(string $s): int
{
    switch ($s) {
        case 'a':
            return 1;
        case 'b':
            return 2;
        default:
            return 0;
    }
}
