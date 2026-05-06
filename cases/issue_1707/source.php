<?php

declare(strict_types=1);

function test(): void
{
    $flag = 0;
    $fail = 0;

    while ($row = mt_rand()) {
        if ($fail) {
            break;
        }

        if (!$flag) {
            $seriesmeta = mt_rand(0, max: 100);
            if (!$seriesmeta) {
                $fail = 1;
            }

            $flag = 1;
        }

        if ($row === mt_rand()) {
            // something
        }
    }
}
