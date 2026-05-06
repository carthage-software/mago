<?php

declare(strict_types=1);

namespace Fixture;

const VALUES = ['Foo', 'Bar'];

$lower = array_map(strtolower(...), VALUES);
$value = strtolower('Foo');

if (in_array($value, $lower, strict: true)) {
    echo 'found';
}
