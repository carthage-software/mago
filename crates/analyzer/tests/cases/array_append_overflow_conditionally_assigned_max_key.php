<?php

declare(strict_types=1);

$a = [0 => 'first'];
if (1 === rand(0, 1)) {
    $a[9223372036854775807] = 'edge';
}

/** @mago-expect analysis:possibly-array-append-overflow */
$a[] = 'over';
