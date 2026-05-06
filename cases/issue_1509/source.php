<?php

declare(strict_types=1);

function search_massupdate(int $commit_after = 1): void
{
    $n = 0;
    $m = 0;

    while (mt_rand(0, max: 10)) {
        if ($m === 0) {
            echo "start_transaction()\n";
        }
        ++$n;
        ++$m;
        if ($m >= $commit_after) {
            echo 'commit()';
            $m = 0;
        }
    }

    if ($m) {
        echo 'commit()';
    }

    if ($n) {
        echo "{$n} total";
    }
}
