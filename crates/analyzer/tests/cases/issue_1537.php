<?php

declare(strict_types=1);

$m = explode(' ', '');
$n = '';

foreach ($m as $s) {
    if (isset($s[0])) {
        $n .= strtoupper($s[0]);
    } else {
        $n .= '?';
    }
    if (isset($s[1])) {
        $n .= strtolower($s[1]);
    }
}
