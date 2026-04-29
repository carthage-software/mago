<?php

declare(strict_types=1);

$a = [
    9223372036854775807 => 'last',
];

/** @mago-expect analysis:array-append-overflow */
$a[] = 'over';
