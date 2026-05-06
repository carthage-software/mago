<?php

/**
 * @return 'one or two'|'three'|'other'
 */
function test_switch_multiple_cases(int $value): string
{
    $result = '';
    switch ($value) {
        case 1:
        case 2:
            $result = 'one or two';
            break;
        case 3:
            $result = 'three';
            break;
        default:
            $result = 'other';
    }
    return $result;
}
