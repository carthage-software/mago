<?php

/**
 * @return 'a'|'bc'|'c'|'d'
 */
function test_switch_mixed_fall_through(int $value): string
{
    $result = '';
    switch ($value) {
        case 1:
            $result .= 'a';
            break;
        case 2:
            $result .= 'b';
        case 3:
            $result .= 'c';
            break;
        default:
            $result .= 'd';
    }
    return $result;
}
