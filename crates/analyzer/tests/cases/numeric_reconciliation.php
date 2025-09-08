<?php

/**
 * @return null|numeric-string
 */
function to_numeric_string(mixed $value): null|string
{
    if (is_string($value) && is_numeric($value)) {
        return $value;
    }

    if (is_numeric($value)) {
        return (string) $value;
    }

    if ($value instanceof Stringable) {
        $str = (string) $value;
        if (is_numeric($str)) {
            return $str;
        }
    }

    return null;
}
