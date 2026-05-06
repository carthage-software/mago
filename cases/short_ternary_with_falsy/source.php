<?php

/**
 * @return 'default'
 *
 */
function short_ternary_with_falsy(): string
{
    $a = '';
    return $a ?: 'default';
}
