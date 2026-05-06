<?php

declare(strict_types=1);

/** @return array<string,int> */
function fassoc(): array
{
    return [
        'object_id' => mt_rand(1, max: 100_000),
        'r' => mt_rand(1, max: 100),
        's' => mt_rand(101, max: 500),
        'year' => mt_rand(2003, max: 2026),
    ];
}

function test(): void
{
    while ($row = fassoc()) {
        $oid = (int) $row['object_id'];
        $r = (int) $row['r'];
        $s = (int) $row['s'];
        $y = (int) $row['year'];

        if ($r !== false) {
            echo "{$oid} in {$r}\n";
        }
        if ($s !== null) {
            echo "{$oid} in {$r}\n";
        }
    }
}
