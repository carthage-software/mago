<?php

declare(strict_types=1);

function flow_switch_fall_through_default(int $code): string
{
    $result = '';

    switch ($code) {
        case 1:
            $result = 'one';
            break;
        case 2:
        case 3:
            $result = 'two-or-three';
            break;
        default:
            $result = 'other';
            break;
    }

    return $result;
}
