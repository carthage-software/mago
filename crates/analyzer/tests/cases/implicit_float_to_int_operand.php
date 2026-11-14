<?php

declare(strict_types=1);

function reported(int $i): void
{
    $f = 1.5;

    /** @mago-expect analysis:implicit-float-to-int-cast */
    echo $f | $i;
    /** @mago-expect analysis:implicit-float-to-int-cast */
    echo $i & $f;
    /** @mago-expect analysis:implicit-float-to-int-cast */
    echo $i >> $f;
    /** @mago-expect analysis:implicit-float-to-int-cast */
    echo ~$f;
    /** @mago-expect analysis:implicit-float-to-int-cast */
    echo 5.5 % 2;
    /** @mago-expect analysis:implicit-float-to-int-cast */
    $i |= $f;
    echo $i;
}

function not_reported(int $i, float $unknown): void
{
    echo 2.0 | $i;
    echo 6.0 % 4;
    echo $unknown & $i;
    echo ~$unknown;
    echo 1.5 + $i;
}
