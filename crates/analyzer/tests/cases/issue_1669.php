<?php

declare(strict_types=1);

function issue_1669_fill_four(): void
{
    $ia = [0, 0, 0, 0];

    while ($val = mt_rand(0, max: 20)) {
        if (!$ia[0]) {
            $ia[0] = $val;
            continue;
        }

        if (!$ia[1]) {
            $ia[1] = $val;
            continue;
        }

        $ia[2] = $ia[3];
        $ia[3] = $val;
    }
}
