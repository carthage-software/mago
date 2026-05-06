<?php

/**
 * @return non-falsy-string
 */
function elvis_operator_with_falsy_string(string $a): string
{
    return $a ?: 'default';
}
