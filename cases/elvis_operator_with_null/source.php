<?php

/**
 * @return 'default'
 *
 */
function elvis_operator_with_null(): string
{
    $a = null;

    return $a ?: 'default';
}
