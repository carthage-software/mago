<?php

/**
 * @return 'is a'|'is b'|'is other'
 */
function test_switch_string_subject(string $value): string
{
    switch ($value) {
        case 'a':
            return 'is a';
        case 'b':
            return 'is b';
        default:
            return 'is other';
    }
}
