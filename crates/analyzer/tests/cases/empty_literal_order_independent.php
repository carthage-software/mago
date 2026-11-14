<?php

declare(strict_types=1);

/**
 * @param numeric-string $n
 *
 * @return ''|numeric-string
 */
function empty_literal_first(string $n, bool $empty): string
{
    return $empty ? '' : $n;
}

/**
 * @param numeric-string $n
 *
 * @return ''|numeric-string
 */
function empty_literal_second(string $n, bool $empty): string
{
    return $empty ? $n : '';
}

/**
 * @param numeric-string $n
 *
 * @return ''|numeric-string
 */
function empty_literal_assigned_first(string $n, bool $flag): string
{
    $v = '';
    if ($flag) {
        $v = $n;
    }

    return $v;
}
