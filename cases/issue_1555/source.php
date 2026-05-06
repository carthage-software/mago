<?php

declare(strict_types=1);

function not_okay(): void
{
    if (mt_rand(0, max: 1)) {
        $a = [mt_rand(0, max: 57)];
        foreach ($a as $k => $v) {
            if ($v === false) {
                unset($a[$k]);
            }
        }
    }
}
