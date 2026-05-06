<?php

/**
 * @assert-if-true !empty $string
 */
function is_non_empty(string $string): bool
{
    return $string !== '';
}

/**
 * @param non-empty-string $str
 */
function i_take_non_empty(string $str): void
{
    echo $str;
}

function i_take_any(string $str): void
{
    if (is_non_empty($str)) {
        i_take_non_empty($str);
    } else {
        i_take_non_empty('default value');
    }
}
