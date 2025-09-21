<?php

/**
 * @param list{'a', 'b'}|list{'b', 'a'} $arr
 *
 * @return list{'b', 'a'}
 */
function x(array $arr): array
{
    if ($arr === ['a', 'b']) {
        return ['b', 'a'];
    } else {
        return $arr;
    }
}
