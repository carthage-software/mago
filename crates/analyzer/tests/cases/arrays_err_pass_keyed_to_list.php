<?php

declare(strict_types=1);

/** @param list<int> $xs */
function takes_list(array $xs): void
{
    foreach ($xs as $x) {
        echo $x;
    }
}

/** @param array<string, int> $arr */
function caller(array $arr): void
{
    // @mago-expect analysis:invalid-argument
    takes_list($arr);
}
