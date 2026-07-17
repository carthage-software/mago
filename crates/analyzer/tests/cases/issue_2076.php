<?php

declare(strict_types=1);

$content = 'blah';
$content2 = 'blahhaha';

try {
    json_decode($content, flags: JSON_THROW_ON_ERROR);
    $a = 0;
    json_decode($content2, flags: JSON_THROW_ON_ERROR);
} catch (\Throwable) {
    if (isset($a)) {
        print_r($a);
    }

    isset($a) && print_r($a);
}
