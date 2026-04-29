<?php

declare(strict_types=1);

$a = [];
$a[PHP_INT_MAX] = 'edge';

/** @mago-expect analysis:array-append-overflow */
$a[] = 'over';
