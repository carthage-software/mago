<?php

/**
 * @return 'abc'|'bc'|'c'
 */
function test_switch_no_break(int $value): string
{
    $result = '';
    switch ($value) {
        case 1:
            $result .= 'a';
        case 2:
            $result .= 'b';
        default:
            $result .= 'c';
    }
    return $result;
}
