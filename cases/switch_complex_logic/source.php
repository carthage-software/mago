<?php

/**
 * @return 'positive one'|'negative one'|'other'
 */
function test_switch_complex_logic(int $value): string
{
    switch ($value) {
        case 1:
            return 'positive one';
        case -1:
            return 'negative one';
        default:
            return 'other';
    }
}
