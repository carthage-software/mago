<?php

declare(strict_types=1);

$ia = [];
$last = 0;
$target = mt_rand(min: 1, max: 5);

for ($i = 0; $i < $target; $i++) {
    $last = $i;
    if (count($ia) < 1) {
        $ia[] = $i;
    }
}

if ($last && !in_array($last, $ia, strict: true)) {
    $ia[1] = $last;
}

// @mago-expect analysis:redundant-condition
if (count($ia)) {
    // @mago-expect analysis:impossible-condition
    echo '0' . (!isset($ia[0]) ? '-' : $ia[0]) . "\n";
    echo '1' . (!isset($ia[1]) ? '-' : $ia[1]) . "\n";
}
