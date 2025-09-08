<?php

/**
 * @assert-if-true empty $string
 */
function is_empty_str(string $string): bool
{
    return $string === '';
}

/**
 * @param '' $str
 */
function i_take_empty_str(string $str): void
{
    echo 'Here comes nothing: ' . $str;
}

/**
 * @param non-empty-string $str
 */
function i_take_non_empty(string $str): void
{
    echo 'Here comes something: ' . $str;
}

function i_take_any(string $str): void
{
    if (is_empty_str($str)) {
        i_take_empty_str($str);
    } else {
        i_take_non_empty($str);
    }
}
