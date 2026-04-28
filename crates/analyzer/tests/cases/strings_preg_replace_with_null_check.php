<?php

declare(strict_types=1);

function probe(string $s): string
{
    $r = preg_replace('/foo/', 'bar', $s);
    if ($r === null) {
        return $s;
    }
    return $r;
}
