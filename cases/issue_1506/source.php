<?php

declare(strict_types=1);

/** @param list<string> $list */
function test(string &$what, array $list)
{
    $types = [];
    $list = ['test', 'test2'];
    foreach ($list as $h) {
        if (!in_array($h, $types, strict: true)) {
            $types[] = $h;
        }
    }

    if (count($types) === 1) {
        $what = $types[0]; // This value has type `null|string('test')|string('test2')`, but the parameter expects `string`.
    }
}

$inout = 'in';
test($inout, ['foo', 'bar']);
