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
 */
function x1(): string
{
    return join_strings(['Hello', ' ', 'World!']);
}

/**
 * @return non-empty-string
 */
function x2(): string
{
    return join_strings(['a' => 'Hello', 'b' => ' ', 'c' => 'World!']);
}
