<?php

/**
 * @param int<0, max> $offset
 *
 * @return null|int<0, max>
 */
function search1(string $haystack, string $needle, int $offset = 0): null|int
{
    if ('' === $needle) {
        return null;
    }

    return false === ($pos = strpos($haystack, $needle, $offset)) ? null : $pos;
}
