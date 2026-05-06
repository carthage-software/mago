<?php

declare(strict_types=1);

namespace App;

class FooBar {}

function processRequest(): void {}

new foobar();
new FOOBAR();
new Foobar();

// Correct, no warning
new FooBar();

processrequest();
PROCESSREQUEST();

// Correct, no warning
processRequest();

FoObAr::class;

new \stdclass();
new \STDCLASS();
// Correct
new \stdClass();

new \arrayobject();
// Correct
new \ArrayObject();

$a = \ARRAY_MAP(fn($x) => $x, []);
$b = \Array_Map(fn($x) => $x, []);
// Correct
$c = \array_map(fn($x) => $x, []);

$d = \Strlen('hello');
// Correct
$e = \strlen('hello');

$f = \Str_Contains('hello', 'lo');
// Correct
$g = \str_contains('hello', 'lo');
