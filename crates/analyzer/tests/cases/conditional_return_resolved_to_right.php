<?php

/**
 * @param non-empty-string[] $strings
 *
 * @return ($strings is non-empty-array ? non-empty-string : string)
 */
function join_strings(array $strings): string
{
    $result = '';
    foreach ($strings as $string) {
        $result .= $string;
    }

    return $result;
}

/**
 * @return non-empty-string
 *
 * @mago-expect analysis:invalid-return-statement
 */
function x1(): string
{
    return join_strings([]);
}
