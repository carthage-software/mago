<?php

/**
 * @return 'hello'
 *
 */
function short_ternary_with_truthy(): string
{
    $a = 'hello';
    return $a ?: 'default';
}
