<?php

$ru = getrusage();
if ($ru) {
    $usec = ($ru['ru_utime.tv_sec'] * 1e6) + $ru['ru_utime.tv_usec'];
    echo (string) $usec;
}
