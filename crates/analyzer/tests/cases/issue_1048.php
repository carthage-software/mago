<?php

declare(strict_types=1);

namespace App;

class FooBar {}
function processRequest(): void {}

/** @mago-expect analysis:incorrect-class-like-casing */
new foobar();
/** @mago-expect analysis:incorrect-class-like-casing */
new FOOBAR();
/** @mago-expect analysis:incorrect-class-like-casing */
new Foobar();

// Correct, no warning
new FooBar();

/** @mago-expect analysis:incorrect-function-casing */
processrequest();
/** @mago-expect analysis:incorrect-function-casing */
PROCESSREQUEST();

// Correct, no warning
processRequest();

/** @mago-expect analysis:incorrect-class-like-casing */
/** @mago-expect analysis:unused-statement */
FoObAr::class;

/** @mago-expect analysis:incorrect-class-like-casing */
new \stdclass();
/** @mago-expect analysis:incorrect-class-like-casing */
new \STDCLASS();
// Correct
new \stdClass();

/** @mago-expect analysis:incorrect-class-like-casing */
new \arrayobject();
// Correct
new \ArrayObject();

/** @mago-expect analysis:incorrect-function-casing */
$a = \ARRAY_MAP(fn($x) => $x, []);
/** @mago-expect analysis:incorrect-function-casing */
$b = \Array_Map(fn($x) => $x, []);
// Correct
$c = \array_map(fn($x) => $x, []);

/** @mago-expect analysis:incorrect-function-casing */
$d = \Strlen('hello');
// Correct
$e = \strlen('hello');

/** @mago-expect analysis:incorrect-function-casing */
$f = \Str_Contains('hello', 'lo');
// Correct
$g = \str_contains('hello', 'lo');
