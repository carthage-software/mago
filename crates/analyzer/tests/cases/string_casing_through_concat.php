<?php

declare(strict_types=1);

/** @param lowercase-string $s */
function string_casing_takes_lower(string $s): void
{
    echo $s;
}

/** @param lowercase-string $lc */
function string_casing_concat(string $lc, int $i): void
{
    string_casing_takes_lower('.' . $lc);
    string_casing_takes_lower($lc . '/');
    string_casing_takes_lower('_' . $i);
    string_casing_takes_lower('a.' . $lc);
    string_casing_takes_lower("a{$lc}");
    string_casing_takes_lower("{$lc}-{$i}");
}

function string_casing_mixed(string $s): void
{
    // @mago-expect analysis:possibly-invalid-argument
    string_casing_takes_lower("a{$s}");
}
