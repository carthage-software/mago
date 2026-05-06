<?php

/**
 * @return 'default'
 */
function test_switch_default_only(int $value): string
{
    $result = '';
    switch ($value) {
        default:
            $result = 'default';
    }
    return $result;
}
