<?php

declare(strict_types=1);

$strings = ['hello', 'world'];
$titled = array_map('ucfirst', $strings);
foreach ($titled as $t) {
    echo $t;
}
