<?php

/**
 * @return 'one'|'two'|'other'
 */
function test_switch_simple_break(int $value): string
{
    $result = '';
    switch ($value) {
        case 1:
            $result = 'one';
            break;
        case 2:
            $result = 'two';
            break;
        default:
            $result = 'other';
            break;
    }
    return $result;
}
