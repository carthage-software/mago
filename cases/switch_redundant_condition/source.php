<?php

/**
 */
function test_switch_redundant_condition(bool $value): string
{
    switch ($value) {
        case true:
            return 'true';
        case false:
            return 'false';
        default:
            return 'unreachable';
    }
}
