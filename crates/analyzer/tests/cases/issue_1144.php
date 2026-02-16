<?php

declare(strict_types=1);

/** @param array{w: int, z: int} $x */
function foo(array $x): void {}

$a = array_map(static fn(int $x): int => 2 * $x, ['w' => 4, 'z' => 4]);

foo($a);

/** @param array{a: string, b: string} $x */
function bar(array $x): void {}

$b = array_map(static fn(int $x): string => (string) $x, ['a' => 1, 'b' => 2]);

bar($b);

/** @param list{string, string, string} $x */
function baz(array $x): void {}

$c = array_map(static fn(int $x): string => (string) $x, [1, 2, 3]);

baz($c);
