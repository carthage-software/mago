<?php

/**
 * @return 'one or two'|'three or default'
 */
function test_switch_empty_case(int $value): string
{
    $result = '';
    switch ($value) {
        case 1:
        case 2:
            $result = 'one or two';
            break;
        case 3:
        default:
            $result = 'three or default';
    }
    return $result;
}
