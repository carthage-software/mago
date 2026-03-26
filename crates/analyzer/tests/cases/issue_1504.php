<?php

declare(strict_types=1);

$n = 0;
$t = '';
$a = [];
for ($i = 0; $i < 1999; $i++) {
    $a[] = $i;
}

foreach ($a as $v) {
    if ($n++ > 1000) {
        break;
    }

    $t .= (string) $v;
}
