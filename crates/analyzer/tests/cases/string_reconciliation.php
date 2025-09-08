<?php

/**
 * @assert-if-true numeric-string $s
 */
function is_numeric_string(string $s): bool
{
    return is_numeric_string($s);
}

/**
 * @param '' $s
 * @return ''
 */
function gimme_empty_string(string $s): string
{
    echo $s;
    return $s;
}

/**
 * @param non-empty-string $s
 * @return non-empty-string
 */
function gimme_non_empty_string(string $s): string
{
    echo $s;
    return $s;
}

/**
 * @param numeric-string $s
 * @return numeric-string
 */
function gimme_numeric_string(string $s): string
{
    echo $s;
    return $s;
}

function str_to_float(string $str): null|float
{
    if ('' === $str) {
        gimme_empty_string($str);

        return 0.0;
    }

    gimme_non_empty_string($str);

    if (is_numeric_string($str)) {
        gimme_numeric_string($str);
        return (float) $str;
    } else {
        gimme_non_empty_string($str);
    }

    return null;
}
