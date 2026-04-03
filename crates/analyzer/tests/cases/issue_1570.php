<?php

declare(strict_types=1);

function gen(): false|string
{
    if (!mt_rand(0, max: 5)) {
        return false;
    }
    return (string) mt_rand(0, max: 1000);
}

function test(float|int $factor): void
{
    $a = [gen()];
    $b = [gen()];

    $p = [];
    foreach ($a as $v1) {
        if ($v1 === false) {
            continue;
        }
        $v1 = (int) $v1;
        foreach ($b as $v2) {
            if ($v2 === false) {
                continue;
            }
            $v2 = (int) $v2;
            $p[] = ($v1 / $v2) * $factor;
        }
    }

    // $p could be empty if all iterations hit `continue`,
    // so count($p) should NOT be flagged as always truthy.
    if (count($p)) {
        echo 'there is something';
    }
}
