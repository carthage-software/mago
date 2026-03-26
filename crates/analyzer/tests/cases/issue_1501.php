<?php

$ru = getrusage();
if ($ru) {
    // @mago-expect analysis:possibly-undefined-array-index,possibly-undefined-array-index
    $usec = ($ru['ru_utime.tv_sec'] * 1e6) + $ru['ru_utime.tv_usec'];
    echo (string) $usec;
}
