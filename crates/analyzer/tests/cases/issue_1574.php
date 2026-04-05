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
        /** @mago-expect analysis:possibly-undefined-string-array-index */
        $oid = (int) $row['object_id'];
        /** @mago-expect analysis:possibly-undefined-string-array-index */
        $r = (int) $row['r'];
        /** @mago-expect analysis:possibly-undefined-string-array-index */
        $s = (int) $row['s'];
        /** @mago-expect analysis:possibly-undefined-string-array-index */
        $y = (int) $row['year'];

        /** @mago-expect analysis:redundant-comparison */
        /** @mago-expect analysis:redundant-condition */
        if ($r !== false) {
            echo "{$oid} in {$r}\n";
        }
        /** @mago-expect analysis:redundant-comparison */
        /** @mago-expect analysis:redundant-condition */
        if ($s !== null) {
            echo "{$oid} in {$r}\n";
        }
    }
}
