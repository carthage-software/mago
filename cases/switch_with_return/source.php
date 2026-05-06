<?php

/**
 * @return 'one'|'two'|'other'
 */
function test_switch_with_return(int $value): string
{
    switch ($value) {
        case 1:
            return 'one';
        case 2:
            return 'two';
        default:
            return 'other';
    }
}
