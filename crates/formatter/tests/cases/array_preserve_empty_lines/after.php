<?php

$result = [
    $qb->where('baz', 'qux')->orderBy('baz'),

    $qb->where('foo', 'bar')->orderBy('foo'),
];

$simple = [
    'first',

    'second',
    'third',

    'fourth',
];

// Multiple empty lines should collapse to one
$multiple = [
    'a',

    'b',
];

// Single element - no separator to preserve
$single = [
    'only',
];

// Inline arrays should not be affected
$inline = ['a', 'b', 'c'];
